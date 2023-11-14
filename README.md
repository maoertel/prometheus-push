# Prometheus push

This crate works as an extension to the [prometheus](https://crates.io/crates/prometheus) crate to be able to push non-blocking (default) to your
Prometheus push gateway and with a less dependent setup of `reqwest` (no `openssl` for example) or with an implementation of your own http client.

By default you have to implement the `Push` trait to use it with your choice of http client or you can use the `with_request` feature.
This feature already implements `Push` in a `PushClient` that leverages `reqwest` under the hood. Reqwest is setup without default features
(minimal set) in this case so it should not interfere with your own applications reqwest setup (e.g. `rust-tls`).

Async functionality is considered the standard in this crate but you can enable the `blocking` feature to get the implementation without async. You
can enable the corresponding blocking `reqwest` implementation with the `with_request_blocking` feature in which case you enable the `blocking`
feature of the `reqwest` crate as well.

## Example with feature `with_reqwest`

```rust
use prometheus::labels;
use prometheus_push::with_request::PushClient;
use prometheus_push::MetricsPusher;
use reqwest::Client;

let push_gateway = "<address to your instance>";
let metrics_pusher = MetricsPusher::<PushClient>::from(Client::new(), &push_gateway);
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
implementation of the `with_request` feature (see [here](https://github.com/maoertel/prometheus-push/blob/7fe1946dd143f4870beb80e642b0acb7854a3cb8/src/with_request.rs)).

Basically it is as simple as that.

```rust
use prometheus_push::Push;

pub struct YourClient {
    ...
}

#[async_trait::async_trait]
impl Push for YourClient {
    async fn push_all(&self, url: &str, body: Vec<u8>, content_type: &str) -> Result<()> {
        // implement a PUT request with your client with this body and `content_type` in header
    }

    async fn push_add(&self, url: &str, body: Vec<u8>, content_type: &str) -> Result<()> {
        // implement a POST request with your client with this body and `content_type` in header
    }
}
```

## Features

- `default`: by default async functionality and no reqwest is enabled
- `with_request`: this feature enables `reqwest` in minimal configuration and enables the alredy implemented `PushClient`
- `blocking`: on top of the default feature you get the same functionality in a blocking fashion
- `with_reqwest_blocking`: like `with_reqwest` but including blocking and completely blocking

## License

[MIT](./LICENSE-MIT)
