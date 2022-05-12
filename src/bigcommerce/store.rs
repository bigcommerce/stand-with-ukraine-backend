use anyhow::Context;
use reqwest::header;
use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct BCStoreInformationResponse {
    pub secure_url: String,
}

pub struct BCStore {
    store_hash: String,
    access_token: Secret<String>,
}

impl BCStore {
    pub fn new(store_hash: String, access_token: Secret<String>) -> Self {
        Self {
            store_hash,
            access_token,
        }
    }

    pub fn get_store_hash(&self) -> &str {
        self.store_hash.as_str()
    }

    pub fn get_access_token(&self) -> &str {
        self.access_token.expose_secret().as_str()
    }

    pub fn get_api_headers(&self) -> Result<header::HeaderMap, anyhow::Error> {
        let mut headers = header::HeaderMap::new();

        headers.insert(
            "X-Auth-Token",
            self.access_token
                .expose_secret()
                .parse()
                .context("Failed to set header value")?,
        );

        Ok(headers)
    }
}
