use actix_web::{dev::ServiceRequest, http::StatusCode, web, Error, HttpResponse, ResponseError};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use secrecy::{ExposeSecret, Secret};
use time::{Duration, OffsetDateTime};

use crate::configuration::JWTSecret;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Claims {
    sub: String,
    role: String,
    exp: i64,
}

pub fn create_jwt<S>(store_id: &str, secret: S) -> Result<String, jsonwebtoken::errors::Error>
where
    S: AsRef<Secret<String>>,
{
    let expiration = OffsetDateTime::now_utc() + Duration::hours(1);

    let claims = Claims {
        sub: store_id.to_owned(),
        role: "user".to_string(),
        exp: expiration.unix_timestamp(),
    };
    let header = Header::new(Algorithm::HS512);
    let key = EncodingKey::from_secret(secret.as_ref().expose_secret().as_bytes());

    encode(&header, &claims, &key)
}

pub struct AuthorizedUser(pub String);

pub fn decode_token<S>(token: &str, secret: S) -> Result<Claims, AuthenticationError>
where
    S: AsRef<Secret<String>>,
{
    let key = DecodingKey::from_secret(secret.as_ref().expose_secret().as_bytes());
    let validation = Validation::new(Algorithm::HS512);
    let decoded = decode::<Claims>(token, &key, &validation).map_err(|e| {
        print!("{}", e);
        AuthenticationError::InvalidTokenError(e.into())
    })?;

    Ok(decoded.claims)
}

pub async fn validate_jwt_bearer_token(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    let jwt_secret = match req.app_data::<web::Data<JWTSecret>>() {
        Some(val) => val,
        None => {
            return Err(Error::from(AuthenticationError::UnexpectedError(
                anyhow::anyhow!("Invalid configuration"),
            )))
        }
    };

    match decode_token(credentials.token(), jwt_secret.as_ref()) {
        Ok(_) => Ok(req),
        Err(err) => Err(Error::from(AuthenticationError::InvalidTokenError(
            err.into(),
        ))),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AuthenticationError {
    #[error("Token is invalid.")]
    InvalidTokenError(#[source] anyhow::Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for AuthenticationError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::UnexpectedError(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
            Self::InvalidTokenError(_) => HttpResponse::new(StatusCode::UNAUTHORIZED),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use rstest::*;

    #[test]
    fn should_encode_and_decode_jwt_in_correct_format() {
        let store_id = "test_store";
        let secret = JWTSecret(Secret::from("abcdefg".to_string()));
        let token = create_jwt(store_id, secret).unwrap();

        let parts: Vec<&str> = token.splitn(3, ".").collect();

        assert!(parts[0].len() > 0);
        assert!(parts[1].len() > 0);
        assert!(parts[2].len() > 0);

        let secret = JWTSecret(Secret::from("abcdefg".to_string()));
        let claims = decode_token(token.as_str(), secret).unwrap();

        assert_eq!("test_store", claims.sub);
        assert!(
            claims.exp > (OffsetDateTime::now_utc() + Duration::minutes(30)).unix_timestamp(),
            "Expiration should be more than 30 mins"
        )
    }
}
