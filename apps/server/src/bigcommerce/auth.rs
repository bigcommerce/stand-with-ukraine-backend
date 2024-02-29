use secrecy::Secret;
use serde::{Deserialize, Serialize};

use super::store::APIToken;

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    pub id: i32,
    pub email: String,
}

#[derive(Deserialize)]
pub struct OAuthResponse {
    pub access_token: Secret<String>,
    pub scope: String,
    pub user: User,
    pub context: String,
}

impl OAuthResponse {
    pub fn get_bigcommerce_store(&self) -> Result<APIToken, anyhow::Error> {
        let store_hash = self
            .context
            .split_once('/')
            .map(|x| x.1)
            .ok_or_else(|| anyhow::anyhow!("Context did not have correct format"))?;

        Ok(APIToken::new(
            store_hash.to_owned(),
            self.access_token.clone(),
        ))
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Claims {
    user: User,
    owner: User,
    sub: String,
}

impl Claims {
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
