use reqwest::Client;

pub struct GerritClient {
    client: Client,
    base_url: String,
    user: String,
    password: String,
}

impl GerritClient {
    pub fn new(base_url: String, user: String, password: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            user,
            password,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}
