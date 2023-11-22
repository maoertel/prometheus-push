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
use crate::with_request::PushClient;
#[cfg(feature = "with_reqwest")]
use reqwest::Client;

#[cfg(feature = "with_reqwest_blocking")]
use crate::blocking::with_request::PushClient;
#[cfg(feature = "with_reqwest_blocking")]
use crate::blocking::MetricsPusher;
#[cfg(feature = "with_reqwest_blocking")]
use crate::blocking::Push;
#[cfg(feature = "with_reqwest_blocking")]
use reqwest::blocking::Client;

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
    /// Creates a new [`MetricsPusher`] with the given [`reqwest::Client`] client and the Url
    /// of your pushgateway instance.
    pub fn from(client: Client, url: &Url) -> Result<PrometheusMetricsPusher> {
        MetricsPusher::new(PushClient::new(client), PrometheusMetricsConverter, url)
    }
}

#[cfg(feature = "with_reqwest_blocking")]
pub type PrometheusMetricsPusherBlocking = MetricsPusher<
    PushClient,
    PrometheusMetricsConverter,
    Vec<MetricFamily>,
    Vec<Box<dyn Collector>>,
    Vec<u8>,
>;

#[cfg(feature = "with_reqwest_blocking")]
impl<P, CM, MF, C, B> MetricsPusher<P, CM, MF, C, B>
where
    P: Push<B>,
    CM: ConvertMetrics<MF, C, B>,
{
    /// Creates a new [`MetricsPusher`] with the given [`reqwest::blocking::Client`] client
    /// and the Url of your pushgateway instance.
    pub fn from(client: Client, url: &Url) -> Result<PrometheusMetricsPusherBlocking> {
        MetricsPusher::new(PushClient::new(client), PrometheusMetricsConverter, url)
    }
}
