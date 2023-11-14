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
use crate::helper::create;
use crate::helper::create_metrics_job_url;
use crate::helper::metric_families_from;
#[cfg(feature = "with_reqwest")]
use crate::with_request::PushClient;

#[async_trait::async_trait]
pub trait Push {
    async fn push_all(&self, url: &Url, body: Vec<u8>, content_type: &str) -> Result<()>;
    async fn push_add(&self, url: &Url, body: Vec<u8>, content_type: &str) -> Result<()>;
}

enum PushType {
    Add,
    All,
}

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

    pub async fn push_all_collectors<'a, BH: BuildHasher>(
        &self,
        job: &'a str,
        grouping: &'a HashMap<&'a str, &'a str, BH>,
        collectors: Vec<Box<dyn Collector>>,
    ) -> Result<()> {
        self.push_collectors(job, grouping, collectors, PushType::All)
            .await
    }

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
        let (url, encoded_metrics, encoder) = create(job, &self.url, grouping, metric_families)?;

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
