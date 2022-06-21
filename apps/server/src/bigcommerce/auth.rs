use secrecy::Secret;

use super::store::BCStore;

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

        Ok(BCStore::new(
            store_hash.to_owned(),
            self.access_token.clone(),
        ))
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct BCClaims {
    user: BCUser,
    owner: BCUser,
    sub: String,
}

impl BCClaims {
    pub fn get_store_hash(&self) -> Result<&str, anyhow::Error> {
        self.sub
            .split_once('/')
            .map(|x| x.1)
            .ok_or_else(|| anyhow::anyhow!("Context did not have correct format"))
    }

    pub fn is_owner(&self) -> bool {
        self.owner.id == self.user.id && self.owner.email == self.user.email
    }
}
