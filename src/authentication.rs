use actix_utils::future::{ready, Ready};
use actix_web::{
    dev::Payload, error::ParseError, http::StatusCode, web, FromRequest, HttpRequest, HttpResponse,
    ResponseError,
};
use actix_web_httpauth::headers::authorization;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use secrecy::{ExposeSecret, Secret};
use time::{Duration, OffsetDateTime};

use crate::configuration::JWTSecret;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct AuthClaims {
    pub sub: String,
    pub role: String,
    pub exp: i64,
}

impl FromRequest for AuthClaims {
    type Future = Ready<Result<Self, Self::Error>>;
    type Error = AuthenticationError;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> <Self as FromRequest>::Future {
        let jwt_secret = match req.app_data::<web::Data<JWTSecret>>() {
            Some(val) => val,
            None => return ready(Err(AuthenticationError::InvalidServerConfiguration)),
        };

        let bearer = match <authorization::Authorization::<authorization::Bearer> as actix_web::http::header::Header>::parse(req) {
            Ok(auth) => auth.into_scheme(),
            Err(err) => {
                return ready(Err(AuthenticationError::NoTokenProvided(err)))
            }
        };

        let claims = match decode_token(bearer.token(), jwt_secret.as_ref()) {
            Ok(claims) => claims,
            Err(err) => return ready(Err(err)),
        };

        ready(Ok(claims))
    }
}

pub fn create_jwt<S>(store_hash: &str, secret: S) -> Result<String, jsonwebtoken::errors::Error>
where
    S: AsRef<Secret<String>>,
{
    let expiration = OffsetDateTime::now_utc() + Duration::hours(1);

    let claims = AuthClaims {
        sub: store_hash.to_owned(),
        role: "user".to_owned(),
        exp: expiration.unix_timestamp(),
    };
    let header = Header::new(Algorithm::HS512);
    let key = EncodingKey::from_secret(secret.as_ref().expose_secret().as_bytes());

    encode(&header, &claims, &key)
}

pub struct AuthorizedUser(pub String);

pub fn decode_token<S>(token: &str, secret: S) -> Result<AuthClaims, AuthenticationError>
where
    S: AsRef<Secret<String>>,
{
    let key = DecodingKey::from_secret(secret.as_ref().expose_secret().as_bytes());
    let validation = Validation::new(Algorithm::HS512);
    let decoded = decode::<AuthClaims>(token, &key, &validation)
        .map_err(AuthenticationError::InvalidTokenError)?;

    Ok(decoded.claims)
}

#[derive(thiserror::Error, Debug)]
pub enum AuthenticationError {
    #[error("No Bearer Token")]
    NoTokenProvided(#[source] ParseError),

    #[error("Token is invalid.")]
    InvalidTokenError(#[source] jsonwebtoken::errors::Error),

    #[error("Server Configuration Invalid")]
    InvalidServerConfiguration,

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for AuthenticationError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::InvalidServerConfiguration => {
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            }
            Self::InvalidTokenError(_) => HttpResponse::new(StatusCode::UNAUTHORIZED),
            Self::NoTokenProvided(_) => HttpResponse::new(StatusCode::UNAUTHORIZED),
            Self::UnexpectedError(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::header::AUTHORIZATION, test::TestRequest, web::Data};

    #[test]
    fn should_encode_and_decode_jwt_in_correct_format() {
        let store_hash = "test_store";
        let secret = JWTSecret(Secret::from("abcdefg".to_owned()));
        let token = create_jwt(store_hash, secret).unwrap();

        let parts: Vec<&str> = token.splitn(3, ".").collect();

        assert!(parts[0].len() > 0);
        assert!(parts[1].len() > 0);
        assert!(parts[2].len() > 0);

        let secret = JWTSecret(Secret::from("abcdefg".to_owned()));
        let claims = decode_token(token.as_str(), secret).unwrap();

        assert_eq!("test_store", claims.sub);
        assert!(
            claims.exp > (OffsetDateTime::now_utc() + Duration::minutes(30)).unix_timestamp(),
            "Expiration should be more than 30 mins"
        )
    }

    #[actix_web::test]
    async fn auth_claims_extractor_fails_without_jwt_secret() {
        let req = TestRequest::default()
            .insert_header((AUTHORIZATION, "test-token"))
            .to_http_request();
        let mut payload = Payload::None;

        let err = AuthClaims::from_request(&req, &mut payload)
            .await
            .unwrap_err();

        assert_eq!(err.to_string(), "Server Configuration Invalid");
    }

    #[actix_web::test]
    async fn auth_claims_extractor_fails_with_no_header() {
        let jwt_secret = Data::new(JWTSecret(Secret::from("test-token".to_owned())));
        let req = TestRequest::default()
            .app_data(jwt_secret.clone())
            .to_http_request();
        let mut payload = Payload::None;

        let err = AuthClaims::from_request(&req, &mut payload)
            .await
            .unwrap_err();

        assert_eq!(err.to_string(), "No Bearer Token");
    }

    #[actix_web::test]
    async fn auth_claims_extractor_fails_with_bad_token() {
        let jwt_secret = Data::new(JWTSecret(Secret::from("test-secret".to_owned())));
        let req = TestRequest::default()
            .app_data(jwt_secret.clone())
            .insert_header((AUTHORIZATION, "Bearer test-token"))
            .to_http_request();
        let mut payload = Payload::None;

        let err = AuthClaims::from_request(&req, &mut payload)
            .await
            .unwrap_err();

        assert_eq!(err.to_string(), "Token is invalid.");
    }
}
