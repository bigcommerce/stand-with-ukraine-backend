use config::{Config, ConfigError, Environment, File};
use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::{
    postgres::{PgConnectOptions, PgSslMode},
    ConnectOptions,
};

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub bigcommerce: BCAppSettings,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub database_name: String,
    pub require_ssl: bool,

    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}

#[derive(serde::Deserialize, Clone)]
pub struct BCAppSettings {
    pub client_id: String,
    pub client_secret: Secret<String>,

    pub api_base_url: String,
    pub login_base_url: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub timeout: u16,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        dbg!(ssl_mode);

        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(self.password.expose_secret())
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        let mut options = self.without_db().database(&self.database_name);
        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    pub base_url: String,
    pub jwt_secret: Secret<String>,

    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let environment: AppEnvironment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    let base_path = std::env::current_dir().expect("Failed to determine the current directory.");
    let configuration_directory = base_path.join("configuration");

    Config::builder()
        .add_source(File::from(configuration_directory.join("base")).required(true))
        .add_source(File::from(configuration_directory.join(environment.as_str())).required(true))
        .add_source(Environment::with_prefix("app").separator("__"))
        .build()?
        .try_deserialize()
}

pub struct ApplicationBaseUrl(pub String);

impl std::fmt::Display for ApplicationBaseUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct JWTSecret(pub Secret<String>);

impl AsRef<Secret<String>> for JWTSecret {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum AppEnvironment {
    Local,
    Production,
}

impl AppEnvironment {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppEnvironment::Local => "local",
            AppEnvironment::Production => "production",
        }
    }
}

impl TryFrom<String> for AppEnvironment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
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
        AppEnvironment::try_from(String::from("unknown")).expect("Should panic");
    }

    #[rstest]
    #[case(AppEnvironment::Local, String::from("local"))]
    #[case(AppEnvironment::Production, String::from("production"))]
    fn try_from_string_into_environment(
        #[case] environment: AppEnvironment,
        #[case] environment_string: String,
    ) {
        assert_eq!(
            environment,
            AppEnvironment::try_from(environment_string)
                .expect("Could not convert string to Environment")
        );
    }
}
