use axum::extract::FromRef;
use axum::RequestPartsExt;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use crate::state::SharedState;

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthClaims {
    pub sub: String,
    pub role: String,
    pub exp: i64,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthClaims
where
    SharedState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    #[tracing::instrument(name = "decode auth from request", skip(parts, state))]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| Error::NoToken)?;

        let state = SharedState::from_ref(state);

        decode_token(bearer.token(), &state.jwt_secret)
    }
}

impl IntoResponse for Error {
    #[tracing::instrument(name = "authentication error")]
    fn into_response(self) -> Response {
        match self {
            Self::InvalidToken(_) | Self::NoToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        }
        .into_response()
    }
}

#[tracing::instrument(name = "create jwt token", skip(secret))]
pub fn create_jwt(
    store_hash: &str,
    secret: &Secret<String>,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = OffsetDateTime::now_utc() + Duration::days(1);

    let claims = AuthClaims {
        sub: store_hash.to_owned(),
        role: "user".to_owned(),
        exp: expiration.unix_timestamp(),
    };
    let header = Header::new(Algorithm::HS512);
    let key = EncodingKey::from_secret(secret.expose_secret().as_bytes());

    encode(&header, &claims, &key)
}

pub struct AuthorizedUser(pub String);

#[tracing::instrument(name = "decode token")]
pub fn decode_token(token: &str, secret: &Secret<String>) -> Result<AuthClaims, Error> {
    let key = DecodingKey::from_secret(secret.expose_secret().as_bytes());
    let mut validation = Validation::new(Algorithm::HS512);
    validation.validate_aud = false;
    let decoded = decode::<AuthClaims>(token, &key, &validation).map_err(Error::InvalidToken)?;

    Ok(decoded.claims)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Token is not provided.")]
    NoToken,

    #[error("Token is invalid.")]
    InvalidToken(#[source] jsonwebtoken::errors::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_encode_and_decode_jwt_in_correct_format() {
        let store_hash = "test_store";
        let secret = Secret::from("abcdefg".to_owned());
        let token = create_jwt(store_hash, &secret).unwrap();

        let parts: Vec<&str> = token.splitn(3, '.').collect();

        assert!(!parts[0].is_empty());
        assert!(!parts[1].is_empty());
        assert!(!parts[2].is_empty());

        let secret = Secret::from("abcdefg".to_owned());
        let claims = decode_token(token.as_str(), &secret).unwrap();

        assert_eq!("test_store", claims.sub);
        assert!(
            claims.exp > (OffsetDateTime::now_utc() + Duration::minutes(30)).unix_timestamp(),
            "Expiration should be more than 30 mins"
        )
    }
}
