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
    pub fn from(client: Client, url: &Url) -> Result<PrometheusClientMetricsPusher> {
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
    pub fn from(
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
