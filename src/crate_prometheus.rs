use std::collections::HashMap;
use std::hash::BuildHasher;

use prometheus::core::Collector;
use prometheus::proto::MetricFamily;
use prometheus::Encoder;
use prometheus::ProtobufEncoder;
use prometheus::Registry;
use url::Url;

use crate::error::LabelType;
use crate::error::PushMetricsError;
use crate::error::Result;
use crate::helper::build_url;
use crate::helper::validate;
use crate::Collect;
use crate::ConvertMetrics;
use crate::MetricFamiliarize;

pub struct PrometheusMetricsConverter;

pub trait MetaCollect: Collect + Collector {}
impl MetricFamiliarize for MetricFamily {}

const LABEL_NAME_JOB: &str = "job";

impl PrometheusMetricsConverter {
    pub fn new() -> Self {
        Self {}
    }

    fn encode_metrics<'a, BH: BuildHasher>(
        &self,
        encoder: &ProtobufEncoder,
        metric_families: Vec<MetricFamily>,
        grouping: &HashMap<&str, &str, BH>,
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

impl<C: MetaCollect + 'static> ConvertMetrics<MetricFamily, C> for PrometheusMetricsConverter {
    fn metric_families_from(&self, collectors: Vec<Box<C>>) -> Result<Vec<MetricFamily>> {
        let registry = Registry::new();
        for collector in collectors {
            registry.register(collector)?;
        }

        Ok(registry.gather())
    }

    fn create_push_details<'a, BH: BuildHasher>(
        &self,
        job: &'a str,
        url: &'a url::Url,
        grouping: &HashMap<&str, &str, BH>,
        metric_families: Vec<MetricFamily>,
    ) -> Result<(Url, Vec<u8>, String)> {
        let url = build_url(url, validate(job)?, grouping)?;
        let encoder = ProtobufEncoder::new();
        let encoded_metrics = self.encode_metrics(&encoder, metric_families, grouping)?;

        Ok((url, encoded_metrics, String::from(encoder.format_type())))
    }
}
