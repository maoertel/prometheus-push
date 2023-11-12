use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use reqwest::Response;
use reqwest::StatusCode;

use crate::error::Result;
use crate::Push;
use crate::PushMetricsError;

pub struct PushClient {
    client: Client,
}

impl PushClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl Push for PushClient {
    async fn push_all(&self, url: &str, body: Vec<u8>, content_type: &str) -> Result<()> {
        let response = &self
            .client
            .put(url)
            .header(CONTENT_TYPE, content_type)
            .body(body)
            .send()
            .await?;

        handle_response(response)
    }

    async fn push_add(&self, url: &str, body: Vec<u8>, content_type: &str) -> Result<()> {
        let response = &self
            .client
            .post(url)
            .header(CONTENT_TYPE, content_type)
            .body(body)
            .send()
            .await?;

        handle_response(response)
    }
}

fn handle_response(response: &Response) -> Result<()> {
    match response.status() {
        StatusCode::ACCEPTED | StatusCode::OK => {
            log::info!("Pushed metrics to the push gateway.");
            Ok(())
        }
        status_code => Err(PushMetricsError::Generic(format!(
            "unexpected status code {status_code} while pushing to {url}",
            url = response.url()
        ))),
    }
}
