use std::sync::Arc;

use config::{Config, ConfigError, Environment, File};
use dotenvy::dotenv;
use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::{
    postgres::{PgConnectOptions, PgSslMode},
    ConnectOptions, PgPool,
};

use crate::{
    bigcommerce::client::HttpAPI as BigCommerceHttpAPI, liq_pay::HttpAPI as LiqPayHttpAPI,
    startup::get_connection_pool,
};

#[derive(serde::Deserialize, Clone)]
pub struct Configuration {
    pub database: Database,
    pub application: Application,
    pub bigcommerce: BigCommerce,
    pub liq_pay: LiqPay,
}

#[derive(serde::Deserialize, Clone)]
pub struct Database {
    pub username: String,
    pub password: Secret<String>,
    pub database_name: String,
    pub require_ssl: bool,

    pub socket: Option<String>,
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}

#[derive(serde::Deserialize, Clone)]
pub struct BigCommerce {
    pub client_id: String,
    pub client_secret: Secret<String>,
    pub install_redirect_uri: String,

    pub api_base_url: String,
    pub login_base_url: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub timeout: u16,
}

impl Database {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        self.socket
            .as_ref()
            .map_or_else(
                || PgConnectOptions::new().host(&self.host).port(self.port),
                |socket| PgConnectOptions::new().socket(socket.as_str()),
            )
            .username(&self.username)
            .password(self.password.expose_secret())
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        let options = self.without_db().database(&self.database_name);
        options.log_statements(tracing::log::LevelFilter::Trace)
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct LiqPay {
    pub public_key: Secret<String>,
    pub private_key: Secret<String>,
}

#[derive(serde::Deserialize, Clone)]
pub struct Application {
    pub base_url: String,
    pub jwt_secret: Secret<String>,

    pub lightstep_access_token: Secret<String>,

    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}

const CONFIGURATION_PATH: &str = "configuration/base";
const SERVER_WORKSPACE_PATH: &str = "apps/server";

impl Configuration {
    /// # Errors
    ///
    /// Will return `ConfigError` if configuration files and environment variables cannot build a valid `Configuration`
    pub fn generate_from_environment() -> Result<Self, ConfigError> {
        dotenv().ok();

        let base_path =
            std::env::current_dir().expect("Failed to determine the current directory.");

        Config::builder()
            .add_source(File::from(base_path.join(CONFIGURATION_PATH)).required(false))
            .add_source(
                File::from(
                    base_path
                        .join(SERVER_WORKSPACE_PATH)
                        .join(CONFIGURATION_PATH),
                )
                .required(false),
            )
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?
            .try_deserialize()
    }

    pub fn get_app_state(&self) -> SharedState {
        let db_pool = get_connection_pool(&self.database);
        let bigcommerce_client = BigCommerceHttpAPI::new(
            self.bigcommerce.api_base_url.clone(),
            self.bigcommerce.login_base_url.clone(),
            self.bigcommerce.client_id.clone(),
            self.bigcommerce.client_secret.clone(),
            self.bigcommerce.install_redirect_uri.clone(),
            std::time::Duration::from_millis(self.bigcommerce.timeout.into()),
        );
        let liq_pay_client = LiqPayHttpAPI::new(
            self.liq_pay.public_key.clone(),
            self.liq_pay.private_key.clone(),
        );

        Arc::new(AppState {
            db_pool,
            base_url: self.application.base_url.clone(),
            jwt_secret: self.application.jwt_secret.clone(),
            bigcommerce_client,
            liq_pay_client,
        })
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub base_url: String,
    pub jwt_secret: Secret<String>,
    pub bigcommerce_client: BigCommerceHttpAPI,
    pub liq_pay_client: LiqPayHttpAPI,
}

pub type SharedState = Arc<AppState>;

pub struct LightstepAccessToken(pub Secret<String>);

impl AsRef<Secret<String>> for LightstepAccessToken {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AppEnvironment {
    Local,
    Production,
}

impl AppEnvironment {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Production => "production",
        }
    }
}

impl TryFrom<&str> for AppEnvironment {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{other} is not a supported environment. Use either `local` or `production`.",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[test]
    #[should_panic]
    fn test_try_from_environment_fail_unknown() {
        AppEnvironment::try_from("unknown").expect("Should panic");
    }

    #[rstest]
    #[case(AppEnvironment::Local, "local")]
    #[case(AppEnvironment::Production, "production")]
    fn try_from_string_into_environment(
        #[case] environment: AppEnvironment,
        #[case] environment_string: &str,
    ) {
        assert_eq!(
            environment,
            AppEnvironment::try_from(environment_string)
                .expect("Could not convert string to Environment")
        );
    }
}
