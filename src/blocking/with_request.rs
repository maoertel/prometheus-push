use reqwest::blocking::Client;
use reqwest::blocking::Response;
use reqwest::header::CONTENT_TYPE;
use reqwest::StatusCode;
use url::Url;

use crate::blocking::Push;
use crate::error::Result;
use crate::helper::handle_response;
use crate::helper::Respond;

pub struct PushClient {
    client: Client,
}

impl PushClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

impl Push for PushClient {
    fn push_all(&self, url: &Url, body: Vec<u8>, content_type: &str) -> Result<()> {
        let response = &self
            .client
            .put(url.as_str())
            .header(CONTENT_TYPE, content_type)
            .body(body)
            .send()?;

        handle_response(response)
    }

    fn push_add(&self, url: &Url, body: Vec<u8>, content_type: &str) -> Result<()> {
        let response = &self
            .client
            .post(url.as_str())
            .header(CONTENT_TYPE, content_type)
            .body(body)
            .send()?;

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
