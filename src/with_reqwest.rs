use reqwest::header::CONTENT_TYPE;
use reqwest::Body;
use reqwest::Client;
use reqwest::Response;
use reqwest::StatusCode;
use url::Url;

use crate::error::Result;
use crate::non_blocking::Push;
use crate::utils::handle_response;
use crate::utils::Respond;

/// `PushClient` is a wrapper for an async `reqwest` http [`Client`] that implements
/// the [`Push`] trait.
#[derive(Debug)]
pub struct PushClient {
    client: Client,
}

impl PushClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<B: Into<Body> + Send + Sync + 'static> Push<B> for PushClient {
    async fn push_all(&self, url: &Url, body: B, content_type: &str) -> Result<()> {
        let response = &self
            .client
            .put(url.as_str())
            .header(CONTENT_TYPE, content_type)
            .body(body)
            .send()
            .await?;

        handle_response(response)
    }

    async fn push_add(&self, url: &Url, body: B, content_type: &str) -> Result<()> {
        let response = &self
            .client
            .post(url.as_str())
            .header(CONTENT_TYPE, content_type)
            .body(body)
            .send()
            .await?;

        handle_response(response)
    }
}

impl Respond for Response {
    fn get_status_code(&self) -> StatusCode {
        self.status()
    }

    fn get_url(&self) -> &Url {
        self.url()
    }
}
