use std::collections::HashMap;

use prometheus_client::collector::Collector;
use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use url::Url;

use crate::error::Result;
use crate::utils::build_url;
use crate::utils::validate;
use crate::ConvertMetrics;

#[cfg(feature = "with_reqwest")]
use crate::non_blocking::MetricsPusher;
#[cfg(feature = "with_reqwest")]
use crate::non_blocking::Push;
#[cfg(feature = "with_reqwest")]
use crate::with_reqwest::PushClient;
#[cfg(feature = "with_reqwest")]
use reqwest::Client;

#[cfg(feature = "with_reqwest_blocking")]
use crate::blocking;

/// `PrometheusClientMetricsConverter` is a [`ConvertMetrics`] implementation that converts
/// the given [`Collector`]s to a [`String`] of metrics that can be pushed to the pushgateway.
pub struct PrometheusClientMetricsConverter;

impl ConvertMetrics<String, Vec<Box<dyn Collector>>, Vec<u8>> for PrometheusClientMetricsConverter {
    fn metrics_from(&self, collectors: Vec<Box<dyn Collector>>) -> Result<String> {
        let mut registry = Registry::default();
        for collector in collectors {
            registry.register_collector(collector);
        }

        let mut writer = String::new();
        encode(&mut writer, &registry)?;
        Ok(writer)
    }

    fn create_push_details(
        &self,
        job: &str,
        url: &Url,
        grouping: &HashMap<&str, &str>,
        metric_families: String,
    ) -> Result<(Url, Vec<u8>, String)> {
        let url = build_url(url, validate(job)?, grouping)?;
        let encoded_metrics = metric_families.into_bytes();

        Ok((url, encoded_metrics, String::from("text/plain")))
    }
}

#[cfg(feature = "with_reqwest")]
pub type PrometheusClientMetricsPusher = MetricsPusher<
    PushClient,
    PrometheusClientMetricsConverter,
    String,
    Vec<Box<dyn Collector>>,
    Vec<u8>,
>;

#[cfg(feature = "with_reqwest")]
impl<P, M, MF, C, B> MetricsPusher<P, M, MF, C, B>
where
    P: Push<B>,
    M: ConvertMetrics<MF, C, B>,
{
    /// Creates a new [`MetricsPusher`] with the given [`Client`] and the [`Url`]
    /// of your pushgateway instance.
    pub fn create(client: Client, url: &Url) -> Result<PrometheusClientMetricsPusher> {
        MetricsPusher::new(
            PushClient::new(client),
            PrometheusClientMetricsConverter,
            url,
        )
    }
}

#[cfg(feature = "with_reqwest_blocking")]
pub type PrometheusClientMetricsPusherBlocking = blocking::MetricsPusher<
    blocking::with_reqwest::PushClient,
    PrometheusClientMetricsConverter,
    String,
    Vec<Box<dyn Collector>>,
    Vec<u8>,
>;

#[cfg(feature = "with_reqwest_blocking")]
impl<P, CM, MF, C, B> blocking::MetricsPusher<P, CM, MF, C, B>
where
    P: blocking::Push<B>,
    CM: ConvertMetrics<MF, C, B>,
{
    /// Creates a new [`MetricsPusher`] with the given [`Client`] and the [`Url`]
    /// of your pushgateway instance.
    pub fn create(
        client: reqwest::blocking::Client,
        url: &Url,
    ) -> Result<PrometheusClientMetricsPusherBlocking> {
        blocking::MetricsPusher::new(
            blocking::with_reqwest::PushClient::new(client),
            PrometheusClientMetricsConverter,
            url,
        )
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use mockito::Mock;
    use mockito::Server;
    use mockito::ServerGuard;
    use prometheus_client::encoding::text::encode;
    use prometheus_client::encoding::EncodeLabelSet;
    use prometheus_client::encoding::EncodeLabelValue;
    use prometheus_client::metrics::counter::Counter;
    use prometheus_client::metrics::family::Family;
    use prometheus_client::registry::Registry;
    use prometheus_client_crate::PrometheusClientMetricsPusher;
    use prometheus_client_crate::PrometheusClientMetricsPusherBlocking;
    use url::Url;

    use crate::prometheus_client_crate;

    #[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue)]
    enum Method {
        #[allow(clippy::upper_case_acronyms)]
        GET,
    }

    #[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
    struct Labels {
        // Use your own enum types to represent label values.
        method: Method,
        // Or just a plain string.
        path: String,
    }

    fn create_metrics() -> String {
        // Given I have a counter metric
        let mut registry = <Registry>::default();
        let http_requests = Family::<Labels, Counter>::default();
        registry.register(
            "http_requests",
            "Number of HTTP requests received",
            http_requests.clone(),
        );
        http_requests
            .get_or_create(&Labels { method: Method::GET, path: "/metrics".to_string() })
            .inc();

        let mut metrics = String::new();
        encode(&mut metrics, &registry).unwrap();

        metrics
    }

    fn create_push_gateway_mock(
        server: &mut ServerGuard,
    ) -> (Mock, Url, &'static str, HashMap<&'static str, &'static str>) {
        let pushgateway_address = Url::parse(&server.url()).unwrap();
        let job = "prometheus_client_crate_job";
        let label_name = "kind";
        let label_value = "test";
        let path = format!("/metrics/job/{job}/{label_name}/{label_value}");

        let grouping: HashMap<&str, &str> = HashMap::from([(label_name, label_value)]);

        let expected = "# HELP http_requests Number of HTTP requests received.\n".to_owned()
            + "# TYPE http_requests counter\n"
            + "http_requests_total{method=\"GET\",path=\"/metrics\"} 1\n"
            + "# EOF\n";

        let pushgateway_mock = server
            .mock("PUT", &*path)
            .with_status(200)
            .match_header("content-type", "text/plain")
            .match_body(mockito::Matcher::from(&*expected))
            .create();

        (pushgateway_mock, pushgateway_address, job, grouping)
    }

    #[cfg(feature = "with_reqwest_blocking")]
    #[test]
    fn test_push_all_blocking_reqwest_prometheus_client_crate() {
        use reqwest::blocking::Client;
        // Given I have a counter metric
        let metrics = create_metrics();

        // And a push gateway and a job
        let mut server = Server::new();
        let (pushgateway_mock, push_gateway_address, job, grouping) =
            create_push_gateway_mock(&mut server);

        // And a blocking prometheus metrics pusher
        let metrics_pusher =
            PrometheusClientMetricsPusherBlocking::create(Client::new(), &push_gateway_address)
                .unwrap();

        // When I push all metrics to the push gateway
        metrics_pusher
            .push_all(job, &grouping, metrics)
            .expect("Failed to push metrics");

        // Then the metrics are received by the push_gateway
        pushgateway_mock.expect(1).assert();
    }

    #[cfg(feature = "with_reqwest")]
    #[tokio::test]
    async fn test_push_all_non_blocking_reqwest_prometheus_client_crate() {
        use reqwest::Client;
        // Given I have a counter metric
        let metrics = create_metrics();

        // And a push gateway and a job
        let mut server = Server::new_async().await;
        let (pushgateway_mock, push_gateway_address, job, grouping) =
            create_push_gateway_mock(&mut server);

        // And a nonblocking prometheus metrics pusher
        let metrics_pusher =
            PrometheusClientMetricsPusher::create(Client::new(), &push_gateway_address).unwrap();

        // When I push all metrics to the push gateway
        metrics_pusher
            .push_all(job, &grouping, metrics)
            .await
            .expect("Failed to push metrics");

        // Then the metrics are received by the push_gateway
        pushgateway_mock.expect(1).assert();
    }
}
