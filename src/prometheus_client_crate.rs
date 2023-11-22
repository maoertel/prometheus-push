use std::collections::HashMap;

use prometheus_client::collector::Collector;
use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use url::Url;

use crate::error::Result;
use crate::utils::build_url;
use crate::utils::validate;
use crate::ConvertMetrics;

pub struct PrometheusMetricsConverter;

impl ConvertMetrics<String, Vec<Box<dyn Collector>>, Vec<u8>> for PrometheusMetricsConverter {
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
