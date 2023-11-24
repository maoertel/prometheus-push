use reqwest::blocking::Body;
use reqwest::blocking::Client;
use reqwest::blocking::Response;
use reqwest::header::CONTENT_TYPE;
use reqwest::StatusCode;
use url::Url;

use crate::blocking::Push;
use crate::error::Result;
use crate::utils::handle_response;
use crate::utils::Respond;

/// `PushClient` is a wrapper for a blocking `reqwest` http [`Client`] that implements
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

impl<B: Into<Body>> Push<B> for PushClient {
    fn push_all(&self, url: &Url, body: B, content_type: &str) -> Result<()> {
        let response = &self
            .client
            .put(url.as_str())
            .header(CONTENT_TYPE, content_type)
            .body(body)
            .send()?;

        handle_response(response)
    }

    fn push_add(&self, url: &Url, body: B, content_type: &str) -> Result<()> {
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
