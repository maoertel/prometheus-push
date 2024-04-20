//! # Prometheus Push
//!
//! `prometheus_push` works as an extension to prometheus crates like [prometheus](https://crates.io/crates/prometheus) or
//! [prometheus-client](https://crates.io/crates/prometheus-client) to be able to push non-blocking (default) or blocking to your Prometheus
//! pushgateway with a less dependent setup of `reqwest` (no `openssl` for example) or with an implementation of your own http client or even
//! another `prometheus` crate – this whole crate is completely generic so you are free to do whatever you want.
//!
//! If you wanna use it with `reqwest`, `prometheus` or `prometheus-client` crates you literally do not have to implement anything (see
//! below), as those common usages are already implemented as features within this crate.
//!
//! In this crates stripped version you have to implement the `Push` trait (see below) to use it with your choice of
//! http client or –as said– you can use the `with_reqwest` or `with_reqwest_blocking` features. These features already implement `Push` in a
//! `PushClient` that leverages `reqwest` under the hood. Reqwest is set up without default features (minimal set) in this case so it should
//! not interfere with your own applications reqwest setup (e.g. `rust-tls`).
//!
//! Async functionality (feature `non_blocking`) is considered the standard in this crate but you can enable the `blocking` feature to get the
//! implementation without async. You can enable the corresponding blocking `reqwest` implementation with the `with_reqwest_blocking` feature in
//! which case you enable the `blocking` feature of the `reqwest` crate.
//!
//! In terms of the underlying prometheus functionality you have to implement the `ConvertMetrics` trait  yourself (see below)
//! or you use the already implemented feature `prometheus_crate` that leverages the [prometheus](https://crates.io/crates/prometheus) crate or
//! `prometheus_client_crate` that uses the [prometheus-client](https://crates.io/crates/prometheus-client) crate.
//!
//! ## Scenarios
//!
//! ### 1. I use `reqwest` and `prometheus` crates in a **non-blocking** fashion
//!
//! In your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! prometheus_push = { version = "<version>", default-features = false, features = ["with_reqwest", "prometheus_crate"] }
//! ```
//!
//! ```ignore
//! use prometheus::labels;
//! use prometheus_push::prometheus_crate::PrometheusMetricsPusher;
//! use reqwest::Client;
//! use url::Url;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!    let push_gateway: Url = Url::parse("<address to pushgateway>")?;
//!    let client = Client::new();
//!    let metrics_pusher = PrometheusMetricsPusher::from(client, &push_gateway)?;
//!    metrics_pusher
//!        .push_all(
//!            "<your push jobs name>",
//!            &labels! { "<label_name>" => "<label_value>" },
//!            prometheus::gather(),
//!         )
//!         .await?;
//!
//!     Ok(())
//! }
//! ```
//!
//!### 2. I use `reqwest` and `prometheus` crates in a **blocking** fashion
//!
//!In your `Cargo.toml`:
//!
//!```toml
//![dependencies]
//!prometheus_push = { version = "<version>", default-features = false, features = ["with_reqwest_blocking", "prometheus_crate"] }
//!```

//!```ignore
//!use prometheus::labels;
//! use prometheus_push::prometheus_crate::PrometheusMetricsPusherBlocking;
//! use reqwest::blocking::Client;
//! use url::Url;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let push_gateway: Url = Url::parse("<address to pushgateway>")?;
//!     let client = Client::new();
//!     let metrics_pusher = PrometheusMetricsPusherBlocking::from(client, &push_gateway)?;
//!     metrics_pusher
//!         .push_all(
//!             "<your push jobs name>",
//!             &labels! { "<label_name>" => "<label_value>" },
//!             prometheus::gather(),
//!         )?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### 3. I use `reqwest` and `prometheus-client` crates in a **non-blocking** fashion
//!
//! In your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! prometheus_push = { version = "<version>", default-features = false, features = ["with_reqwest", "prometheus_client_crate"] }
//! ```

//! ```ignore
//! use prometheus_client::encoding::text::encode;
//! use prometheus_push::prometheus_client_crate::PrometheusClientMetricsPusher;
//! use reqwest::Client;
//! use url::Url;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let push_gateway: Url = Url::parse("<address to pushgateway>")?;
//!     let client = Client::new();
//!     let metrics_pusher = PrometheusClientMetricsPusher::from(client, &push_gateway)?;
//!     let grouping: HashMap<&str, &str> = HashMap::from([("<label_name>", "<label_value>")]);
//!     let mut metrics = String::new();
//!     encode(&mut metrics, &registry)?;
//!
//!     metrics_pusher
//!         .push_all(
//!             "<your push jobs name>",
//!             &grouping,
//!             metrics,
//!         )
//!         .await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### 4. I use `reqwest` and `prometheus-client` crates in a **blocking** fashion
//!
//! In your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! prometheus_push = { version = "<version>", default-features = false, features = ["with_reqwest_blocking", "prometheus_client_crate"] }
//! ```
//!
//! ```ignore
//! use prometheus_client::encoding::text::encode;
//! use prometheus_push::prometheus_client_crate::PrometheusClientMetricsPusherBlocking;
//! use reqwest::blocking::Client;
//! use url::Url;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let push_gateway: Url = Url::parse("<address to pushgateway>")?;
//!     let client = Client::new();
//!     let metrics_pusher = PrometheusClientMetricsPusherBlocking::from(client, &push_gateway)?;
//!     let grouping: HashMap<&str, &str> = HashMap::from([("<label_name>", "<label_value>")]);
//!     let mut metrics = String::new();
//!     encode(&mut metrics, &registry)?;
//!
//!     metrics_pusher
//!         .push_all(
//!             "<your push jobs name>",
//!             &grouping,
//!             metrics,
//!         )?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### 5. I want to implement everything myself
//!
//! In case you wanna implement everything yourself you can do so by implementing the `Push` trait and the `ConvertMetrics` trait.
//!
//! #### Implement `Push` yourself
//!
//! If you are not using reqwest as an http client you are free to implement the `Push` traits two methods yourself. As a guide you can use the
//! implementation of the `with_reqwest` feature (see [here](https://github.com/maoertel/prometheus-push/blob/7fe1946dd143f4870beb80e642b0acb7854a3cb8/src/with_reqwest.rs)).
//! Basically it is as simple as that.
//!
//! ```ignore
//! use prometheus_push::Push;
//!
//! pub struct YourPushClient;
//!
//! #[async_trait::async_trait]
//! impl Push<Vec<u8>> for YourPushClient {
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
//! #### Implement `ConvertMetrics` yourself
//!
//! In case you want to use another prometheus client implementation you can implement your own type that implements
//! the `ConvertMetrics` trait to inject it into your instance of `MetricsPusher`.
//!
//! ```ignore
//! impl ConvertMetrics<Vec<YourMetricFamily>, Vec<Box<dyn YourCollector>>, Vec<u8>> for YourMetricsConverter {
//!     fn metric_families_from(
//!         &self,
//!         collectors: Vec<Box<dyn YourCollector>>,
//!     ) -> Result<Vec<YourMetricFamily>> {
//!         // implement the conversion from your Collectors to your MetricsFamilies, or whatever
//!         // your generic `MF` type stands for
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
//! - `prometheus_client_crate`: enables the functionality of the [prometheus-client](https://crates.io/crates/prometheus-client) crate
//!

#[cfg(feature = "blocking")]
pub mod blocking;
#[cfg(feature = "non_blocking")]
pub mod non_blocking;
#[cfg(feature = "prometheus_client_crate")]
pub mod prometheus_client_crate;
#[cfg(feature = "prometheus_crate")]
pub mod prometheus_crate;
#[cfg(feature = "with_reqwest")]
pub mod with_reqwest;

pub mod error;
mod utils;

use std::collections::HashMap;

use url::Url;

use crate::error::Result;

/// `ConvertMetrics` defines the interface for the implementation of your own prometheus logic
/// to incorporate it into [`non_blocking::MetricsPusher`] or [`blocking::MetricsPusher`].
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
