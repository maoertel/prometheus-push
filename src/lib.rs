#[cfg(feature = "blocking")]
pub mod blocking;
pub mod error;
mod helper;
#[cfg(feature = "with_reqwest")]
pub mod with_request;

use std::collections::HashMap;
use std::hash::BuildHasher;

use prometheus::core::Collector;
use prometheus::proto::MetricFamily;
use prometheus::Encoder;
#[cfg(feature = "with_reqwest")]
use reqwest::Client;
use url::Url;

use crate::error::Result;
use crate::helper::create_metrics_job_url;
use crate::helper::create_push_details;
use crate::helper::metric_families_from;
#[cfg(feature = "with_reqwest")]
use crate::with_request::PushClient;

/// Push is a trait that defines the interface for the implementation of your own http
/// client of choice.
#[async_trait::async_trait]
pub trait Push {
    async fn push_all(&self, url: &Url, body: Vec<u8>, content_type: &str) -> Result<()>;
    async fn push_add(&self, url: &Url, body: Vec<u8>, content_type: &str) -> Result<()>;
}

enum PushType {
    Add,
    All,
}

/// MetricsPusher is a prometheus push gateway client that holds information about the
/// address of your push gateway instance and the [`Push`] client that is used to push
/// metrics to the push gateway.
#[derive(Debug)]
pub struct MetricsPusher<P: Push> {
    push_client: P,
    url: Url,
}

impl<P: Push> MetricsPusher<P> {
    pub fn new(push_client: P, url: &Url) -> Result<MetricsPusher<P>> {
        let url = create_metrics_job_url(url)?;
        Ok(Self { push_client, url })
    }

    #[cfg(feature = "with_reqwest")]
    pub fn from(client: Client, url: &Url) -> Result<MetricsPusher<PushClient>> {
        MetricsPusher::new(PushClient::new(client), url)
    }

    /// Pushes all metrics to your push gateway instance.
    ///
    /// Job name and grouping labels must not contain the character '/'.
    ///
    /// As this method pushes all metrics to the push gateway it replaces all previously
    /// pushed metrics with the same job and grouping labels.
    pub async fn push_all<'a, BH: BuildHasher>(
        &self,
        job: &'a str,
        grouping: &'a HashMap<&'a str, &'a str, BH>,
        metric_families: Vec<MetricFamily>,
    ) -> Result<()> {
        self.push(job, grouping, metric_families, PushType::All)
            .await
    }

    /// Pushes all metrics to your push gateway instance with add logic. It will only replace
    /// recently pushed metrics with the same name and grouping labels.
    ///
    /// Job name and grouping labels must not contain the character '/'.
    pub async fn push_add<'a, BH: BuildHasher>(
        &self,
        job: &'a str,
        grouping: &'a HashMap<&'a str, &'a str, BH>,
        metric_families: Vec<MetricFamily>,
    ) -> Result<()> {
        self.push(job, grouping, metric_families, PushType::Add)
            .await
    }

    /// Pushes all metrics from collectors to the push gateway.
    pub async fn push_all_collectors<'a, BH: BuildHasher>(
        &self,
        job: &'a str,
        grouping: &'a HashMap<&'a str, &'a str, BH>,
        collectors: Vec<Box<dyn Collector>>,
    ) -> Result<()> {
        self.push_collectors(job, grouping, collectors, PushType::All)
            .await
    }

    /// Pushes all metrics from collectors to the push gateway with add logic. It will only replace
    /// recently pushed metrics with the same name and grouping labels.
    pub async fn push_add_collectors<'a, BH: BuildHasher>(
        &self,
        job: &'a str,
        grouping: &'a HashMap<&'a str, &'a str, BH>,
        collectors: Vec<Box<dyn Collector>>,
    ) -> Result<()> {
        self.push_collectors(job, grouping, collectors, PushType::Add)
            .await
    }

    async fn push_collectors<'a, BH: BuildHasher>(
        &self,
        job: &'a str,
        grouping: &'a HashMap<&'a str, &'a str, BH>,
        collectors: Vec<Box<dyn Collector>>,
        push_type: PushType,
    ) -> Result<()> {
        let metric_families = metric_families_from(collectors)?;
        self.push(job, grouping, metric_families, push_type).await
    }

    async fn push<'a, BH: BuildHasher>(
        &self,
        job: &'a str,
        grouping: &'a HashMap<&'a str, &'a str, BH>,
        metric_families: Vec<MetricFamily>,
        push_type: PushType,
    ) -> Result<()> {
        let (url, encoded_metrics, encoder) =
            create_push_details(job, &self.url, grouping, metric_families)?;

        match push_type {
            PushType::Add => {
                self.push_client
                    .push_add(&url, encoded_metrics, encoder.format_type())
                    .await
            }

            PushType::All => {
                self.push_client
                    .push_all(&url, encoded_metrics, encoder.format_type())
                    .await
            }
        }
    }
}
