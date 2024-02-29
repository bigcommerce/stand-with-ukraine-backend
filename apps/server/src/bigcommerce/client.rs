use anyhow::Context;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use reqwest::{header, Client};
use secrecy::{ExposeSecret, Secret};
use serde_json::json;

use crate::authentication::Error;

use super::{
    auth::{Claims, OAuthResponse},
    script::{GetResponse, ListResponse, Script},
    store::{APIToken, Information},
};

#[derive(Clone)]
pub struct HttpAPI {
    api_base_url: String,
    login_base_url: String,
    client_id: String,
    client_secret: Secret<String>,
    install_redirect_uri: String,
    http_client: Client,
}

impl HttpAPI {
    pub fn new(
        api_base_url: String,
        login_base_url: String,
        client_id: String,
        client_secret: Secret<String>,
        install_redirect_uri: String,
        timeout: std::time::Duration,
    ) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "Content-Type",
            header::HeaderValue::from_static("application/json"),
        );
        let http_client = Client::builder()
            .timeout(timeout)
            .default_headers(headers)
            .build()
            .expect("BigCommerce api client could not be built");

        Self {
            api_base_url,
            login_base_url,
            client_id,
            client_secret,
            install_redirect_uri,
            http_client,
        }
    }

    fn get_oauth2_url(&self) -> String {
        format!("{}/oauth2/token", self.login_base_url)
    }

    pub async fn authorize_oauth_install(
        &self,
        code: &str,
        scope: &str,
        context: &str,
    ) -> Result<OAuthResponse, reqwest::Error> {
        self.http_client
            .post(self.get_oauth2_url())
            .json(&json!({
                "client_id": self.client_id,
                "client_secret": self.client_secret.expose_secret(),
                "redirect_uri": self.install_redirect_uri,
                "grant_type": "authorization_code",
                "code": code,
                "scope": scope,
                "context": context
            }))
            .send()
            .await?
            .json()
            .await
    }

    fn get_scripts_route(&self, store_hash: &str) -> String {
        format!(
            "{}/stores/{}/v3/content/scripts",
            self.api_base_url, store_hash
        )
    }

    fn get_store_information_route(&self, store_hash: &str) -> String {
        format!("{}/stores/{}/v2/store", self.api_base_url, store_hash)
    }

    fn get_scripts_route_with_id(&self, store_hash: &str, script_id: &str) -> String {
        format!("{}/{}", self.get_scripts_route(store_hash), script_id)
    }

    pub async fn get_all_scripts(&self, store: &APIToken) -> Result<ListResponse, anyhow::Error> {
        self.http_client
            .get(self.get_scripts_route(store.get_store_hash()))
            .headers(store.get_api_headers()?)
            .send()
            .await
            .context("get all scripts request")?
            .error_for_status()?
            .json::<ListResponse>()
            .await
            .context("parse get all scripts response")
    }

    pub async fn try_get_script_with_name(
        &self,
        store: &APIToken,
        name: &str,
    ) -> Result<Option<GetResponse>, anyhow::Error> {
        let scripts = self.get_all_scripts(store).await?;

        for script in scripts.data {
            if script.name == name {
                return Ok(Some(script));
            }
        }

        Ok(None)
    }

    pub async fn remove_all_scripts(&self, store: &APIToken) -> Result<(), anyhow::Error> {
        let scripts = self.get_all_scripts(store).await?;

        for script in scripts.data {
            self.http_client
                .delete(self.get_scripts_route_with_id(store.get_store_hash(), &script.uuid))
                .headers(store.get_api_headers()?)
                .send()
                .await
                .context("delete script request")?;
        }

        Ok(())
    }

    pub async fn create_script(
        &self,
        store: &APIToken,
        script: &Script,
    ) -> Result<(), anyhow::Error> {
        self.http_client
            .post(self.get_scripts_route(store.get_store_hash()))
            .headers(store.get_api_headers()?)
            .json(&script.generate_script_body())
            .send()
            .await
            .context("create script request")?
            .error_for_status()?;

        Ok(())
    }

    pub async fn update_script(
        &self,
        store: &APIToken,
        script_uuid: &str,
        script: &Script,
    ) -> Result<(), anyhow::Error> {
        self.http_client
            .put(self.get_scripts_route_with_id(store.get_store_hash(), script_uuid))
            .headers(store.get_api_headers()?)
            .json(&script.generate_script_body())
            .send()
            .await
            .context("update script request")?
            .error_for_status()?;

        Ok(())
    }

    pub fn decode_jwt(&self, token: &str) -> Result<Claims, Error> {
        let key = DecodingKey::from_secret(self.client_secret.expose_secret().as_bytes());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&[&self.client_id]);

        let decoded = decode::<Claims>(token, &key, &validation).map_err(Error::InvalidToken)?;

        Ok(decoded.claims)
    }

    pub async fn get_store_information(
        &self,
        store: &APIToken,
    ) -> Result<Information, anyhow::Error> {
        self.http_client
            .get(self.get_store_information_route(store.get_store_hash()))
            .headers(store.get_api_headers()?)
            .send()
            .await
            .context("get store information request")?
            .error_for_status()?
            .json::<Information>()
            .await
            .context("parse store information response")
    }
}
