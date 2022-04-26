use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde_json::json;

use crate::authentication::AuthenticationError;

pub struct BCClient {
    api_base_url: String,
    login_base_url: String,
    client_id: String,
    client_secret: Secret<String>,
    http_client: Client,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct BCUser {
    pub id: i32,
    pub email: String,
}

#[derive(serde::Deserialize)]
pub struct BCOAuthResponse {
    pub access_token: Secret<String>,
    pub scope: String,
    pub user: BCUser,
    pub context: String,
}

impl BCOAuthResponse {
    pub fn get_bigcommerce_store(&self) -> Result<BCStore, anyhow::Error> {
        let store_hash = self
            .context
            .split_once('/')
            .map(|x| x.1)
            .ok_or_else(|| anyhow::anyhow!("Context did not have correct format"))?;

        Ok(BCStore {
            store_hash: store_hash.to_string(),
            access_token: self.access_token.expose_secret().to_string(),
        })
    }
}

impl BCClient {
    pub fn new(
        api_base_url: String,
        login_base_url: String,
        client_id: String,
        client_secret: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();

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
        callback_url: &str,
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

    fn get_scripts_route_with_id(&self, store_hash: &str, script_id: &str) -> String {
        format!("{}/{}", self.get_scripts_route(store_hash), script_id)
    }

    pub async fn remove_scripts(&self, store: &BCStore) -> Result<(), anyhow::Error> {
        let BCListScriptsResponse { data } = self
            .http_client
            .get(self.get_scripts_route(&store.store_hash))
            .header("X-Auth-Token", &store.access_token)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!(e))?
            .error_for_status()?
            .json::<BCListScriptsResponse>()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        for script in data {
            self.http_client
                .delete(self.get_scripts_route_with_id(&store.store_hash, &script.uuid))
                .header("X-Auth-Token", &store.access_token)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!(e))?;
        }

        Ok(())
    }

    pub async fn create_script(
        &self,
        store: &BCStore,
        script_content: &String,
    ) -> Result<(), anyhow::Error> {
        self
            .http_client
            .post(self.get_scripts_route(&store.store_hash))
            .header("X-Auth-Token", &store.access_token)
            .json(&json!({
                    "name": "Stand With Ukraine",
                    "description": "This script displays the stand with ukraine widget on your storefront. Configure it from the Stand With Ukraine app installed on your store.",
                    "kind": "script_tag",
                    "html": script_content,
                    "load_method": "default",
                    "location": "footer",
                    "visibility": "storefront",
                    "consent_category": "essential",
                    "auto_uninstall": true,
                    "enabled": true,
            }))
            .send()
            .await.map_err(|e| anyhow::anyhow!(e))?.error_for_status()?;

        Ok(())
    }

    pub fn decode_jwt(&self, token: &str) -> Result<BCClaims, AuthenticationError> {
        let key = DecodingKey::from_secret(self.client_secret.expose_secret().as_bytes());
        let validation = Validation::new(Algorithm::HS256);
        let decoded = decode::<BCClaims>(token, &key, &validation)
            .map_err(|e| AuthenticationError::InvalidTokenError(e.into()))?;

        Ok(decoded.claims)
    }
}

#[derive(serde::Deserialize)]
pub struct BCListScriptsResponse {
    pub data: Vec<BCScript>,
}

#[derive(serde::Deserialize)]
pub struct BCScript {
    pub uuid: String,
    pub api_client_id: String,
    pub enabled: bool,
    pub channel_id: i16,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct BCClaims {
    pub user: BCUser,
    pub owner: BCUser,
    pub sub: String,
}

impl BCClaims {
    pub fn get_store_hash(&self) -> Result<&str, anyhow::Error> {
        self.sub
            .split_once('/')
            .map(|x| x.1)
            .ok_or_else(|| anyhow::anyhow!("Context did not have correct format"))
    }
}

#[derive(serde::Serialize)]
pub struct BCStore {
    pub store_hash: String,
    pub access_token: String,
}
