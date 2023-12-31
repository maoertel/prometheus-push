# Prometheus Push

`prometheus_push` works as an extension to prometheus crates like [prometheus](https://crates.io/crates/prometheus) to be able to push non-blocking (default)
or blocking to your Prometheus pushgateway and with a less dependent setup of `reqwest` (no `openssl` for example) or with an implementation of your
own http client.

By default you have to implement the `Push` trait to use it with your choice of http client or you can use the `with_reqwest` feature.
This feature already implements `Push` in a `PushClient` that leverages `reqwest` under the hood. Reqwest is setup without default features
(minimal set) in this case so it should not interfere with your own applications reqwest setup (e.g. `rust-tls`).

Async functionality is considered the standard in this crate but you can enable the `blocking` feature to get the implementation without async. You
can enable the corresponding blocking `reqwest` implementation with the `with_reqwest_blocking` feature in which case you enable the `blocking`
feature of the `reqwest` crate.

In terms of the underlying prometheus functionality you have to implement the `ConvertMetrics` trait or you use the already implemented feature
`prometheus_crate` that leverages the [prometheus](https://crates.io/crates/prometheus) crate
or `prometheus_client_crate` that uses the [prometheus-client](https://crates.io/crates/prometheus-client) crate.

## Example with features `with_reqwest` and `prometheus_crate`

```rust
use prometheus::labels;
use prometheus_push::prometheus_crate::PrometheusMetricsPusher;
use reqwest::Client;
use url::Url;

let push_gateway: Url = "<address to your instance>";
let client = Client::new();
let metrics_pusher = PrometheusMetricsPusher::from(client, &push_gateway)?;
metrics_pusher
  .push_all(
    "<your push jobs name>",
    &labels! { "<label_name>" => "<label_value>" },
    prometheus::gather(),
  )
  .await?;
```

## Implement `Push` yourself

If you are not using reqwest as an http client you are free to implement the `Push` traits two methods yourself. As a guide you can use the
implementation of the `with_reqwest` feature (see [here](https://github.com/maoertel/prometheus-push/blob/7fe1946dd143f4870beb80e642b0acb7854a3cb8/src/with_reqwest.rs)).
Basically it is as simple as that.

```rust
use prometheus_push::non_blocking::Push;

pub struct YourPushClient;

#[async_trait::async_trait]
impl Push<Vec<u8>> for YourPushClient {
    async fn push_all(&self, url: &Url, body: Vec<u8>, content_type: &str) -> Result<()> {
        // implement a PUT request with your client with this body and `content_type` in header
    }

    async fn push_add(&self, url: &Url, body: Vec<u8>, content_type: &str) -> Result<()> {
        // implement a POST request with your client with this body and `content_type` in header
    }
}
```

## Implement `ConvertMetrics` yourself

In case you want to use another prometheus client implementation you can implement your own type that implements
the `ConvertMetrics` trait to inject it into your instance of `MetricsPusher`.

```rust
impl ConvertMetrics<Vec<YourMetricFamily>, Vec<Box<dyn YourCollector>>, Vec<u8>> for YourMetricsConverter {
    fn metric_families_from(
        &self,
        collectors: Vec<Box<dyn YourCollector>>,
    ) -> Result<Vec<YourMetricFamily>> {
        // implement the conversion from your Collectors to your MetricsFamilies, or whatever
        // your generic `MF` type stands for
    }

    fn create_push_details(
        &self,
        job: &str,
        url: &Url,
        grouping: &HashMap<&str, &str>,
        metric_families: Vec<YourMetricFamily>,
    ) -> Result<(Url, Vec<u8>, String)> {
        // create your push details for the `Push` methods: Url, body and content type
    }
}
```
## Features

- `default`: by default async functionality and no reqwest is enabled
- `non_blocking`: this ennables the async functionality
- `blocking`: on top of the default feature you get the same functionality in a blocking fashion
- `with_reqwest`: this feature enables the `non_blocking` feature as well as `reqwest` in minimal configuration and enables the alredy implemented `PushClient`
- `with_reqwest_blocking`: like `with_reqwest` but including `blocking` instead of `non_blocking`
- `prometheus_crate`: enables the functionality of the [prometheus](https://crates.io/crates/prometheus) crate
- `prometheus_client_crate`: enables the functionality of the [prometheus-client](https://crates.io/crates/prometheus-client) crate

## Integration in your `Cargo.toml`

Let's say you wanna use it with `reqwest` in an async fashion with the `prometheus` crate, you have to add the following to your `Cargo.toml`:

```toml
[dependencies]
prometheus_push = { version = "<version>", default-features = false, features = ["with_reqwest", "prometheus_crate"] }
```

## License

[MIT](./LICENSE-MIT)
