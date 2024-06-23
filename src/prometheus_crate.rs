use std::collections::HashMap;

use prometheus::core::Collector;
use prometheus::proto::MetricFamily;
use prometheus::Encoder;
use prometheus::ProtobufEncoder;
use prometheus::Registry;
use url::Url;

use crate::error::LabelType;
use crate::error::PushMetricsError;
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

/// `PrometheusMetricsConverter` is a [`ConvertMetrics`] implementation that converts
/// the given [`Collector`]s to a [`Vec`] of [`MetricFamily`] that can be used to be
/// pushed to the pushgateway.
pub struct PrometheusMetricsConverter;

const LABEL_NAME_JOB: &str = "job";

impl ConvertMetrics<Vec<MetricFamily>, Vec<Box<dyn Collector>>, Vec<u8>>
    for PrometheusMetricsConverter
{
    fn metrics_from(&self, collectors: Vec<Box<dyn Collector>>) -> Result<Vec<MetricFamily>> {
        let registry = Registry::new();
        for collector in collectors {
            registry.register(collector)?;
        }

        Ok(registry.gather())
    }

    fn create_push_details(
        &self,
        job: &str,
        url: &Url,
        grouping: &HashMap<&str, &str>,
        metric_families: Vec<MetricFamily>,
    ) -> Result<(Url, Vec<u8>, String)> {
        let url = build_url(url, validate(job)?, grouping)?;
        let encoder = ProtobufEncoder::new();
        let encoded_metrics = self.encode_metrics(&encoder, metric_families, grouping)?;

        Ok((url, encoded_metrics, String::from(encoder.format_type())))
    }
}

impl PrometheusMetricsConverter {
    fn encode_metrics(
        &self,
        encoder: &ProtobufEncoder,
        metric_families: Vec<MetricFamily>,
        grouping: &HashMap<&str, &str>,
    ) -> Result<Vec<u8>> {
        let mut encoded_metrics = Vec::new();
        for metric_family in metric_families {
            for metric in metric_family.get_metric() {
                for label_pair in metric.get_label() {
                    let label_name = label_pair.get_name();

                    if LABEL_NAME_JOB == label_name {
                        return Err(PushMetricsError::contains_label(
                            metric_family.get_name(),
                            LabelType::Job,
                        ));
                    }

                    if grouping.contains_key(label_name) {
                        return Err(PushMetricsError::contains_label(
                            metric_family.get_name(),
                            LabelType::Grouping(label_name),
                        ));
                    }
                }
            }

            encoder.encode(&[metric_family], &mut encoded_metrics)?;
        }

        Ok(encoded_metrics)
    }
}

#[cfg(feature = "with_reqwest")]
pub type PrometheusMetricsPusher = MetricsPusher<
    PushClient,
    PrometheusMetricsConverter,
    Vec<MetricFamily>,
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
    pub fn from(client: Client, url: &Url) -> Result<PrometheusMetricsPusher> {
        MetricsPusher::new(PushClient::new(client), PrometheusMetricsConverter, url)
    }
}

#[cfg(feature = "with_reqwest_blocking")]
pub type PrometheusMetricsPusherBlocking = blocking::MetricsPusher<
    blocking::with_reqwest::PushClient,
    PrometheusMetricsConverter,
    Vec<MetricFamily>,
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
    pub fn from(
        client: reqwest::blocking::Client,
        url: &Url,
    ) -> Result<PrometheusMetricsPusherBlocking> {
        blocking::MetricsPusher::new(
            blocking::with_reqwest::PushClient::new(client),
            PrometheusMetricsConverter,
            url,
        )
    }
}

#[cfg(test)]
#[cfg(feature = "with_reqwest")]
mod test {
    use mockito::Server;
    use prometheus::labels;
    use prometheus::Counter;
    use prometheus::Encoder;
    use prometheus::Opts;
    use prometheus::ProtobufEncoder;
    use prometheus_crate::PrometheusMetricsPusher;
    use reqwest::Client;
    use url::Url;

    use crate::prometheus_crate;

    #[tokio::test]
    async fn test_push_all_non_blocking_reqwest_prometheus_crate() {
        // Given I have a counter metric
        let counter_opts = Opts::new("test_counter", "test counter help");
        let counter = Counter::with_opts(counter_opts).unwrap();
        prometheus::register(Box::new(counter.clone())).unwrap();
        counter.inc();

        let metric_families = prometheus::gather();
        let mut metrics = vec![];

        // A push gateway and a job
        let mut server = Server::new_async().await;
        server.reset();
        let push_gateway_address = Url::parse(&server.url()).unwrap();
        let job = "prometheus_crate_job";
        let label_name = "kind";
        let label_value = "test";
        let path = format!("/metrics/job/{job}/{label_name}/{label_value}");

        let encoder = ProtobufEncoder::new();
        encoder.encode(&metric_families, &mut metrics).unwrap();

        let pushgateway_mock = server
            .mock("PUT", &*path)
            .with_status(200)
            .match_header("content-type", encoder.format_type())
            .match_body(mockito::Matcher::from(metrics))
            .create();

        // And a nonblocking prometheus metrics pusher
        let metrics_pusher =
            PrometheusMetricsPusher::from(Client::new(), &push_gateway_address).unwrap();

        // When I push all metrics to the push gateway
        metrics_pusher
            .push_all(job, &labels! { label_name => label_value }, metric_families)
            .await
            .expect("Failed to push metrics");

        // Then the metrics are received by the push_gateway
        pushgateway_mock.expect(1).assert();
    }
}
