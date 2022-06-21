use anyhow::Context;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use reqwest::{header, Client};
use secrecy::{ExposeSecret, Secret};
use serde_json::json;

use crate::{authentication::Error, configuration::BaseURL};

use super::{
    auth::{BCClaims, BCOAuthResponse},
    script::{BCListScriptsResponse, BCScriptResponse, Script},
    store::{BCStore, BCStoreInformationResponse},
};

pub struct BCClient {
    api_base_url: String,
    login_base_url: String,
    client_id: String,
    client_secret: Secret<String>,
    http_client: Client,
}

impl BCClient {
    pub fn new(
        api_base_url: String,
        login_base_url: String,
        client_id: String,
        client_secret: Secret<String>,
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
            .unwrap();

        Self {
            api_base_url,
            login_base_url,
            client_id,
            client_secret,
            http_client,
        }
    }

    fn get_oauth2_url(&self) -> String {
        format!("{}/oauth2/token", self.login_base_url)
    }

    pub async fn authorize_oauth_install(
        &self,
        callback_url: &BaseURL,
        code: &str,
        scope: &str,
        context: &str,
    ) -> Result<BCOAuthResponse, reqwest::Error> {
        let callback_url = format!("{}/bigcommerce/install", callback_url);

        self.http_client
            .post(self.get_oauth2_url())
            .json(&json!({
                "client_id": self.client_id,
                "client_secret": self.client_secret.expose_secret(),
                "redirect_uri": callback_url,
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

    pub async fn get_all_scripts(
        &self,
        store: &BCStore,
    ) -> Result<BCListScriptsResponse, anyhow::Error> {
        self.http_client
            .get(self.get_scripts_route(store.get_store_hash()))
            .headers(store.get_api_headers()?)
            .send()
            .await
            .context("get all scripts request")?
            .error_for_status()?
            .json::<BCListScriptsResponse>()
            .await
            .context("parse get all scripts response")
    }

    pub async fn try_get_script_with_name(
        &self,
        store: &BCStore,
        name: &str,
    ) -> Result<Option<BCScriptResponse>, anyhow::Error> {
        let scripts = self.get_all_scripts(store).await?;

        for script in scripts.data {
            if script.name == name {
                return Ok(Some(script));
            }
        }

        Ok(None)
    }

    pub async fn remove_all_scripts(&self, store: &BCStore) -> Result<(), anyhow::Error> {
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
        store: &BCStore,
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
        store: &BCStore,
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

    pub fn decode_jwt(&self, token: &str) -> Result<BCClaims, Error> {
        let key = DecodingKey::from_secret(self.client_secret.expose_secret().as_bytes());
        let validation = Validation::new(Algorithm::HS256);
        let decoded =
            decode::<BCClaims>(token, &key, &validation).map_err(Error::InvalidTokenError)?;

        Ok(decoded.claims)
    }

    pub async fn get_store_information(
        &self,
        store: &BCStore,
    ) -> Result<BCStoreInformationResponse, anyhow::Error> {
        self.http_client
            .get(self.get_store_information_route(store.get_store_hash()))
            .headers(store.get_api_headers()?)
            .send()
            .await
            .context("get store information request")?
            .error_for_status()?
            .json::<BCStoreInformationResponse>()
            .await
            .context("parse store information response")
    }
}
