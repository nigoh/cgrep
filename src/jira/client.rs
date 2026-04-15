use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use reqwest::Client;

pub struct JiraClient {
    client: Client,
    base_url: String,
    auth_header: String,
}

impl JiraClient {
    pub fn new(base_url: String, user: String, token: String) -> Self {
        let credentials = B64.encode(format!("{user}:{token}"));
        Self {
            client: Client::new(),
            base_url,
            auth_header: format!("Basic {credentials}"),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn auth_header(&self) -> &str {
        &self.auth_header
    }
}
