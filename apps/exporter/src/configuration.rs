use config::{Config, ConfigError, Environment, File};
use dotenvy::dotenv;
use serde::Deserialize;
use swu_app::configuration::Database;

#[derive(Deserialize, Clone)]
pub struct Configuration {
    pub database: Database,
    pub sheets: Sheets,
}

#[derive(Deserialize, Clone)]
pub struct Sheets {
    pub spreadsheet_id: String,
    pub credential_path: String,
    pub token_cache_path: String,
}

impl Configuration {
    pub fn generate_from_environment() -> Result<Self, ConfigError> {
        dotenv().ok();

        let base_path =
            std::env::current_dir().expect("Failed to determine the current directory.");
        let configuration_directory = base_path.join("configuration");

        Config::builder()
            .add_source(File::from(configuration_directory.join("base")).required(true))
            .add_source(Environment::with_prefix("EXPORTER").separator("__"))
            .build()?
            .try_deserialize()
    }
}
