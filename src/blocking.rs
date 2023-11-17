#[cfg(feature = "with_reqwest_blocking")]
pub mod with_request;

use std::collections::HashMap;
use std::hash::BuildHasher;

#[cfg(feature = "with_reqwest_blocking")]
use reqwest::blocking::Client;
use url::Url;

#[cfg(feature = "with_reqwest_blocking")]
use crate::blocking::with_request::PushClient;
use crate::error::Result;
use crate::helper::create_metrics_job_url;
use crate::ConvertMetrics;
use crate::PushType;

/// Push is a trait that defines the interface for the implementation of your own http
/// client of choice.
pub trait Push {
    fn push_all(&self, url: &Url, body: Vec<u8>, content_type: &str) -> Result<()>;
    fn push_add(&self, url: &Url, body: Vec<u8>, content_type: &str) -> Result<()>;
}

/// MetricsPusher is a prometheus pushgateway client that holds information about the
/// address of your pushgateway instance and the [`Push`] client that is used to push
/// metrics to the pushgateway.
#[derive(Debug)]
pub struct MetricsPusher<P, M, MF, C>
where
    P: Push,
    M: ConvertMetrics<MF, C>,
{
    push_client: P,
    metrics_converter: M,
    url: Url,
    mf: std::marker::PhantomData<MF>,
    c: std::marker::PhantomData<C>,
}

impl<P, M, MF, C> MetricsPusher<P, M, MF, C>
where
    P: Push,
    M: ConvertMetrics<MF, C>,
{
    pub fn new(
        push_client: P,
        metrics_converter: M,
        url: &Url,
    ) -> Result<MetricsPusher<P, M, MF, C>> {
        let url = create_metrics_job_url(url)?;
        Ok(Self {
            push_client,
            metrics_converter,
            url,
            mf: std::marker::PhantomData,
            c: std::marker::PhantomData,
        })
    }

    #[cfg(feature = "with_reqwest_blocking")]
    pub fn from(
        client: Client,
        metrics_worker: M,
        url: &Url,
    ) -> Result<MetricsPusher<PushClient, M, MF, C>> {
        MetricsPusher::new(PushClient::new(client), metrics_worker, url)
    }

    /// Pushes all metrics to your pushgateway instance.
    ///
    /// Job name and grouping labels must not contain the character '/'.
    ///
    /// As this method pushes all metrics to the pushgateway it replaces all previously
    /// pushed metrics with the same job and grouping labels.
    pub fn push_all<BH: BuildHasher>(
        &self,
        job: &str,
        grouping: &HashMap<&str, &str, BH>,
        metric_families: Vec<MF>,
    ) -> Result<()> {
        self.push(job, grouping, metric_families, PushType::All)
    }

    /// Pushes all metrics to your pushgateway instance with add logic. It will only replace
    /// recently pushed metrics with the same name and grouping labels.
    ///
    /// Job name and grouping labels must not contain the character '/'.
    pub fn push_add<BH: BuildHasher>(
        &self,
        job: &str,
        grouping: &HashMap<&str, &str, BH>,
        metric_families: Vec<MF>,
    ) -> Result<()> {
        self.push(job, grouping, metric_families, PushType::Add)
    }

    pub fn push_all_collectors<BH: BuildHasher>(
        &self,
        job: &str,
        grouping: &HashMap<&str, &str, BH>,
        collectors: Vec<C>,
    ) -> Result<()> {
        self.push_collectors(job, grouping, collectors, PushType::All)
    }

    /// Pushes all metrics from collectors to the pushgateway.
    pub fn push_add_collectors<BH: BuildHasher>(
        &self,
        job: &str,
        grouping: &HashMap<&str, &str, BH>,
        collectors: Vec<C>,
    ) -> Result<()> {
        self.push_collectors(job, grouping, collectors, PushType::Add)
    }

    /// Pushes all metrics from collectors to the pushgateway with add logic. It will only replace
    /// recently pushed metrics with the same name and grouping labels.
    fn push_collectors<BH: BuildHasher>(
        &self,
        job: &str,
        grouping: &HashMap<&str, &str, BH>,
        collectors: Vec<C>,
        push_type: PushType,
    ) -> Result<()> {
        let metric_families = self.metrics_converter.metric_families_from(collectors)?;
        self.push(job, grouping, metric_families, push_type)
    }

    fn push<BH: BuildHasher>(
        &self,
        job: &str,
        grouping: &HashMap<&str, &str, BH>,
        metric_families: Vec<MF>,
        push_type: PushType,
    ) -> Result<()> {
        let (url, encoded_metrics, encoder) = self.metrics_converter.create_push_details(
            job,
            &self.url,
            grouping,
            metric_families,
        )?;

        match push_type {
            PushType::Add => self.push_client.push_add(&url, encoded_metrics, &encoder),
            PushType::All => self.push_client.push_all(&url, encoded_metrics, &encoder),
        }
    }
}
