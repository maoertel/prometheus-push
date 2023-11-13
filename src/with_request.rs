use crate::error::Result;
use crate::helper::handle_response;
use crate::helper::Respond;
use crate::Push;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use reqwest::Response;
use reqwest::StatusCode;

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

impl Respond for Response {
    fn get_status_code(&self) -> StatusCode {
        self.status()
    }

    fn get_url(&self) -> &reqwest::Url {
        self.url()
    }
}
