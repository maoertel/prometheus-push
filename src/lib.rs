pub mod error;
#[cfg(feature = "with_reqwest")]
pub mod reqwest;

use std::collections::HashMap;
use std::hash::BuildHasher;

use prometheus::proto::MetricFamily;
use prometheus::Encoder;
use prometheus::ProtobufEncoder;

use crate::error::PushMetricsError;
use crate::error::Result;

const LABEL_NAME_JOB: &str = "job";

#[async_trait::async_trait]
pub trait Push {
    async fn push_all(&self, url: &str, body: Vec<u8>, content_type: &str) -> Result<()>;
    async fn push_add(&self, url: &str, body: Vec<u8>, content_type: &str) -> Result<()>;
}

enum PushType {
    Add,
    All,
}

pub struct MetricsPusher<P: Push> {
    pusher: P,
    url: String,
}

impl<P: Push> MetricsPusher<P> {
    pub fn new(pusher: P, url: &str) -> MetricsPusher<P> {
        let mut slash = "/";
        let mut scheme = "http://";
        if url.contains("://") {
            scheme = ""
        };
        if url.ends_with('/') {
            slash = ""
        };

        let url = format!("{scheme}{url}{slash}metrics/job");

        Self { pusher, url }
    }
    pub async fn push_all<'a, BH: BuildHasher>(
        &self,
        job: &'a str,
        grouping: &'a HashMap<&'a str, &'a str, BH>,
        metric_families: Vec<MetricFamily>,
    ) -> Result<()> {
        self.push(job, grouping, metric_families, PushType::All)
            .await
    }

    pub async fn push_add<'a, BH: BuildHasher>(
        &self,
        job: &'a str,
        grouping: &'a HashMap<&'a str, &'a str, BH>,
        metric_families: Vec<MetricFamily>,
    ) -> Result<()> {
        self.push(job, grouping, metric_families, PushType::Add)
            .await
    }

    async fn push<'a, BH: BuildHasher>(
        &self,
        job: &'a str,
        grouping: &'a HashMap<&'a str, &'a str, BH>,
        metric_families: Vec<MetricFamily>,
        push_type: PushType,
    ) -> Result<()> {
        if job.contains('/') {
            return Err(PushMetricsError::Generic(format!(
                "job name must not contain '/': {job}"
            )));
        }

        let mut url_params = vec![job];
        for (label_name, label_value) in grouping {
            if label_value.contains('/') {
                return Err(PushMetricsError::Generic(format!(
                    "value of grouping label {label_name} contains '/': {label_value}",
                )));
            }
            url_params.push(label_name);
            url_params.push(label_value);
        }

        let url = format!(
            "{url}/{params}",
            url = self.url,
            params = url_params.join("/")
        );

        let encoder = ProtobufEncoder::new();
        let mut encoded_metrics = Vec::new();

        for metric_family in metric_families {
            for metric in metric_family.get_metric() {
                for label_pair in metric.get_label() {
                    let label_name = label_pair.get_name();

                    if let LABEL_NAME_JOB = label_name {
                        return Err(PushMetricsError::Generic(format!(
                            "pushed metric {metric} already contains a job label",
                            metric = metric_family.get_name()
                        )));
                    }

                    if grouping.contains_key(label_name) {
                        return Err(PushMetricsError::Generic(format!(
                            "pushed metric {metric} already contains grouping label {label_name}",
                            metric = metric_family.get_name(),
                        )));
                    }
                }
            }

            encoder.encode(&[metric_family], &mut encoded_metrics)?;
        }

        match push_type {
            PushType::Add => {
                self.pusher
                    .push_add(&url, encoded_metrics, encoder.format_type())
                    .await
            }
            PushType::All => {
                self.pusher
                    .push_all(&url, encoded_metrics, encoder.format_type())
                    .await
            }
        }
    }
}
