//! # Prometheus push
//!
//! This crate works as an extension to prometheus crates like [prometheus](https://crates.io/crates/prometheus) to be able to push non-blocking (default)
//! or blocking to your Prometheus pushgateway and with a less dependent setup of `reqwest` (no `openssl` for example) or with an implementation of your
//! own http client.
//!
//! By default you have to implement the `Push` trait to use it with your choice of http client or you can use the `with_reqwest` feature.
//! This feature already implements `Push` in a `PushClient` that leverages `reqwest` under the hood. Reqwest is setup without default features
//! (minimal set) in this case so it should not interfere with your own applications reqwest setup (e.g. `rust-tls`).
//!
//! Async functionality is considered the standard in this crate but you can enable the `blocking` feature to get the implementation without async. You
//! can enable the corresponding blocking `reqwest` implementation with the `with_reqwest_blocking` feature in which case you enable the `blocking`
//! feature of the `reqwest` crate as well.
//!
//! In terms of the underlying prometheus functionality you have to implement the `ConvertMetrics` trait or you use the already implemented feature
//! `prometheus_crate` that leverages the [prometheus](https://crates.io/crates/prometheus) crate.
//!
//! ## Example with features `with_reqwest` and `prometheus_crate`
//!
//! ```compile_fail
//! use prometheus::core::Collector;
//! use prometheus::labels;
//! use prometheus::proto::MetricFamily;
//! use prometheus_push::non_blocking::MetricsPusher;
//! use prometheus_push::prometheus_crate::PrometheusMetricsConverter;
//! use prometheus_push::with_reqwest::PushClient;
//! use prometheus_push::MetricsPusher;
//! use reqwest::Client;
//! use url::Url;
//!
//! pub type PrometheusMetricsPusher =
//!   MetricsPusher<PushClient, PrometheusMetricsConverter, MetricFamily, Box<dyn Collector>>;
//!
//! let push_gateway: Url = <address to your instance>;
//! let client = Client::new();
//! let metrics_pusher = PrometheusMetricsPusher::from(client, &push_gateway)?;
//! metrics_pusher
//!   .push_all(
//!     "<your push jobs name>",
//!     &labels! { "<label_name>" => "<label_value>" },
//!     prometheus::gather(),
//!   )
//!   .await?;
//! ```
//!
//! ## Implement `Push` yourself
//!
//! If you are not using reqwest as an http client you are free to implement the `Push` traits two methods yourself. As a guide you can use the
//! implementation of the `with_reqwest` feature (see [here](https://github.com/maoertel/prometheus-push/blob/7fe1946dd143f4870beb80e642b0acb7854a3cb8/src/with_reqwest.rs)).
//!
//! Basically it is as simple as that.
//!
//! ```compile_fail
//! use prometheus_push::Push;
//! ...
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
//! ## Implement `ConvertMetrics` yourself
//!
//! In case you want to use another promethues client implementation you can implement your own type that implements
//! the `ConvertMetrics` trait to inject it into your instance of `MetricsPusher`.
//!
//! ```compile_fail
//! impl ConvertMetrics<YourMetricFamily, Box<dyn YourCollector>> for YourMetricsConverter {
//!     fn metric_families_from(
//!         &self,
//!         collectors: Vec<Box<dyn YourCollector>>,
//!     ) -> Result<Vec<YourMetricFamily>> {
//!         // implement the conversion from your Collectors to your MetricsFamilies
//!     }
//!
//!     fn create_push_details(
//!         &self,
//!         job: &str,
//!         url: &Url,
//!         grouping: &HashMap<&str, &str>,
//!         metric_families: Vec<YourMetricFamily>,
//!     ) -> Result<(Url, Vec<u8>, String)> {
//!         // create your push details for the `Push` methods: Url, body and content type
//!     }
//! }
//! ```
//!
//! ## Features
//!
//! - `default`: by default async functionality and no reqwest is enabled
//! - `non_blocking`: this ennables the async functionality
//! - `blocking`: on top of the default feature you get the same functionality in a blocking fashion
//! - `with_reqwest`: this feature enables the `non_blocking` feature as well as `reqwest` in minimal configuration and enables the alredy implemented `PushClient`
//! - `with_reqwest_blocking`: like `with_reqwest` but including `blocking` instead of `non_blocking`
//! - `prometheus_crate`: enables the functionality of the [prometheus](https://crates.io/crates/prometheus) crate
//!

#[cfg(all(feature = "with_request", feature = "with_reqwest_blocking"))]
compile_error!("Feature 'with_request' and 'with_reqwest_blocking' are mutually exclusive and cannot be enabled together");

#[cfg(feature = "blocking")]
pub mod blocking;
#[cfg(feature = "non_blocking")]
pub mod non_blocking;
#[cfg(feature = "prometheus_client_crate")]
pub mod prometheus_client_crate;
#[cfg(feature = "prometheus_crate")]
pub mod prometheus_crate;
#[cfg(feature = "with_reqwest")]
pub mod with_request;

pub mod error;
mod utils;

use std::collections::HashMap;

use url::Url;

use crate::error::Result;

/// ConvertMetrics defines the interface for the implementation of your own prometheus logic
/// to incorporate it into [`MetricsPusher`].
pub trait ConvertMetrics<MF, C, B> {
    /// metric_families_from converts the given collectors to metric families.
    fn metrics_from(&self, collectors: C) -> Result<MF>;

    /// create_push_details creates the input arguments for the [`Push`] clients methods.
    fn create_push_details(
        &self,
        job: &str,
        url: &Url,
        grouping: &HashMap<&str, &str>,
        metrics: MF,
    ) -> Result<(Url, B, String)>;
}
