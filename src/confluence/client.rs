use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use reqwest::Client;

// ────────────────────────────────────────────────────────────────────────────
// ConfluenceClient
//
// Holds a reusable `reqwest::Client`, the Confluence base URL, and a
// pre-formatted `Authorization: Basic …` header value.
// ────────────────────────────────────────────────────────────────────────────

pub struct ConfluenceClient {
    client: Client,
    base_url: String,
    auth_header: String,
}

impl ConfluenceClient {
    /// Create a new client.
    ///
    /// * `base_url` – e.g. `https://confluence.example.com`
    /// * `user`     – Atlassian account e-mail / username
    /// * `token`    – API token (or password for Server)
    pub fn new(base_url: String, user: String, token: String) -> Self {
        let credentials = B64.encode(format!("{user}:{token}"));
        Self {
            client: Client::new(),
            base_url,
            auth_header: format!("Basic {credentials}"),
        }
    }

    /// The Confluence instance root URL (no trailing slash guaranteed).
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// The underlying `reqwest` HTTP client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// The `Authorization` header value (`"Basic <base64>"`).
    pub fn auth_header(&self) -> &str {
        &self.auth_header
    }
}
