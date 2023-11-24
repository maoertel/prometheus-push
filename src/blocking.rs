#[cfg(feature = "with_reqwest_blocking")]
pub mod with_reqwest;

use std::collections::HashMap;

use url::Url;

use crate::error::Result;
use crate::utils::create_metrics_job_url;
use crate::utils::PushType;
use crate::ConvertMetrics;

/// `MetricsPusher` is a prometheus pushgateway client that holds information about the
/// address of your pushgateway instance and the [`Push`] client that is used to push
/// metrics to the pushgateway. Furthermore it needs a [`ConvertMetrics`] implementation
/// that converts the metrics to the format that is used by the pushgateway.
#[derive(Debug)]
pub struct MetricsPusher<P, CM, MF, C, B>
where
    P: Push<B>,
    CM: ConvertMetrics<MF, C, B>,
{
    push_client: P,
    metrics_converter: CM,
    url: Url,
    mf: std::marker::PhantomData<MF>,
    c: std::marker::PhantomData<C>,
    b: std::marker::PhantomData<B>,
}

/// `Push` is a trait that defines the interface for the implementation of your own http
/// client of choice.
pub trait Push<B> {
    fn push_all(&self, url: &Url, body: B, content_type: &str) -> Result<()>;
    fn push_add(&self, url: &Url, body: B, content_type: &str) -> Result<()>;
}

impl<P, CM, MF, C, B> MetricsPusher<P, CM, MF, C, B>
where
    P: Push<B>,
    CM: ConvertMetrics<MF, C, B>,
{
    /// Creates a new [`MetricsPusher`] with the given [`Push`] client, [`ConvertMetrics`]
    /// implementation and the url of your pushgateway instance.
    pub fn new(
        push_client: P,
        metrics_converter: CM,
        url: &Url,
    ) -> Result<MetricsPusher<P, CM, MF, C, B>> {
        let url = create_metrics_job_url(url)?;
        Ok(Self {
            push_client,
            metrics_converter,
            url,
            mf: std::marker::PhantomData,
            c: std::marker::PhantomData,
            b: std::marker::PhantomData,
        })
    }

    /// Pushes all metrics to your pushgateway instance.
    ///
    /// Job name and grouping labels must not contain the character '/'.
    ///
    /// As this method pushes all metrics to the pushgateway it replaces all previously
    /// pushed metrics with the same job and grouping labels.
    pub fn push_all(
        &self,
        job: &str,
        grouping: &HashMap<&str, &str>,
        metric_families: MF,
    ) -> Result<()> {
        self.push(job, grouping, metric_families, PushType::All)
    }

    /// Pushes all metrics to your pushgateway instance with add logic. It will only replace
    /// recently pushed metrics with the same name and grouping labels.
    ///
    /// Job name and grouping labels must not contain the character '/'.
    pub fn push_add(
        &self,
        job: &str,
        grouping: &HashMap<&str, &str>,
        metric_families: MF,
    ) -> Result<()> {
        self.push(job, grouping, metric_families, PushType::Add)
    }

    pub fn push_all_collectors(
        &self,
        job: &str,
        grouping: &HashMap<&str, &str>,
        collectors: C,
    ) -> Result<()> {
        self.push_collectors(job, grouping, collectors, PushType::All)
    }

    /// Pushes all metrics from collectors to the pushgateway.
    pub fn push_add_collectors(
        &self,
        job: &str,
        grouping: &HashMap<&str, &str>,
        collectors: C,
    ) -> Result<()> {
        self.push_collectors(job, grouping, collectors, PushType::Add)
    }

    /// Pushes all metrics from collectors to the pushgateway with add logic. It will only replace
    /// recently pushed metrics with the same name and grouping labels.
    fn push_collectors(
        &self,
        job: &str,
        grouping: &HashMap<&str, &str>,
        collectors: C,
        push_type: PushType,
    ) -> Result<()> {
        let metric_families = self.metrics_converter.metrics_from(collectors)?;
        self.push(job, grouping, metric_families, push_type)
    }

    fn push(
        &self,
        job: &str,
        grouping: &HashMap<&str, &str>,
        metric_families: MF,
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
