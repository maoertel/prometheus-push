//! This crate works as an extension to the [prometheus](https://crates.io/crates/prometheus) crate to be able to push non-blocking (default) to your
//! Prometheus pushgateway and with a less dependent setup of `reqwest` (no `openssl` for example) or with an implementation of your own http client.

//! By default you have to implement the `Push` trait to use it with your choice of http client or you can use the `with_reqwest` feature.
//! This feature already implements `Push` in a `PushClient` that leverages `reqwest` under the hood. Reqwest is setup without default features
//! (minimal set) in this case so it should not interfere with your own applications reqwest setup (e.g. `rust-tls`).
//!
//! Async functionality is considered the standard in this crate but you can enable the `blocking` feature to get the implementation without async. You
//! can enable the corresponding blocking `reqwest` implementation with the `with_reqwest_blocking` feature in which case you enable the `blocking`
//! feature of the `reqwest` crate as well.
//!
//! ## Example with feature `with_reqwest`
//!
//! ```compile_fail
//! use prometheus::labels;
//! use prometheus_push::with_reqwest::PushClient;
//! use prometheus_push::MetricsPusher;
//! use reqwest::Client;
//! use url::Url;

//! let push_gateway: Url = "<address to your instance>";
//! let metrics_pusher = MetricsPusher::<PushClient>::from(Client::new(), &push_gateway)?;
//! metrics_pusher
//!   .push_all(
//!     "<your push jobs name>",
//!     &labels! { "<label_name>" => "<label_value>" },
//!     prometheus::gather(),
//!   )
//!   .await?;
//! ```
//! ## Implement `Push` yourself
//!
//! If you are not using reqwest as an http client you are free to implement the `Push` traits two methods yourself. As a guide you can use the
//! implementation of the `with_reqwest` feature (see [here](https://github.com/maoertel/prometheus-push/blob/7fe1946dd143f4870beb80e642b0acb7854a3cb8/src/with_reqwest.rs)).
//!
//! Basically it is as simple as that.
//!
//! ```compile_fail
//! use prometheus_push::error::Result;
//! use prometheus_push::Push;
//! use url::Url;
//!
//! pub struct YourClient {
//!     ...
//! }
//!
//! #[async_trait::async_trait]
//! impl Push for YourClient {
//!     async fn push_all(&self, url: &Url, body: Vec<u8>, content_type: &str) -> Result<()> {
//!         // implement a PUT request with your client with this body and `content_type` in header
//!     }
//!
//!     async fn push_add(&self, url: &Url, body: Vec<u8>, content_type: &str) -> Result<()> {
//!         // implement a POST request with your client with this body and `content_type` in header
//!     }
//! }
//! ```
//!
//! ## Features
//!
//! - `default`: by default async functionality and no reqwest is enabled
//! - `with_reqwest`: this feature enables `reqwest` in minimal configuration and enables the alredy implemented `PushClient`
//! - `blocking`: on top of the default feature you get the same functionality in a blocking fashion
//! - `with_reqwest_blocking`: like `with_reqwest` but including blocking and completely blocking
//!

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

/// MetricsPusher is a prometheus pushgateway client that holds information about the
/// address of your pushgateway instance and the [`Push`] client that is used to push
/// metrics to the pushgateway.
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

    /// Pushes all metrics to your pushgateway instance.
    ///
    /// Job name and grouping labels must not contain the character '/'.
    ///
    /// As this method pushes all metrics to the pushgateway it replaces all previously
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

    /// Pushes all metrics to your pushgateway instance with add logic. It will only replace
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

    /// Pushes all metrics from collectors to the pushgateway.
    pub async fn push_all_collectors<'a, BH: BuildHasher>(
        &self,
        job: &'a str,
        grouping: &'a HashMap<&'a str, &'a str, BH>,
        collectors: Vec<Box<dyn Collector>>,
    ) -> Result<()> {
        self.push_collectors(job, grouping, collectors, PushType::All)
            .await
    }

    /// Pushes all metrics from collectors to the pushgateway with add logic. It will only replace
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
