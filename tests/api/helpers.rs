use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use once_cell::sync::Lazy;
use secrecy::{ExposeSecret, Secret};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use swu_app::{
    authentication::{create_jwt, AuthClaims},
    configuration::{get_configuration, DatabaseSettings, JWTSecret},
    startup::{get_connection_pool, Application},
    telemetry::{get_subscriber, init_subscriber},
};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;
use wiremock::MockServer;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "debug".into();
    let subscriber_name = "test".into();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,

    pub bigcommerce_server: MockServer,
    pub jwt_secret: JWTSecret,
    pub base_url: String,
    pub bc_secret: Secret<String>,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let bigcommerce_server = MockServer::start().await;

    // configuration for this test instance
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;

        // we can reuse the mock server for both for now
        c.bigcommerce.api_base_url = bigcommerce_server.uri();
        c.bigcommerce.login_base_url = bigcommerce_server.uri();
        c
    };

    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();
    let _ = tokio::spawn(application.run_until_stopped());

    let test_app = TestApp {
        address: format!("http://127.0.0.1:{}", application_port),
        port: application_port,
        bigcommerce_server,
        db_pool: get_connection_pool(&configuration.database),
        jwt_secret: JWTSecret(configuration.application.jwt_secret),
        bc_secret: Secret::from(configuration.bigcommerce.client_secret),
        base_url: configuration.application.base_url,
    };

    test_app
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}

impl TestApp {
    pub fn generate_bc_jwt_token(&self) -> String {
        let exp = ((OffsetDateTime::now_utc() + Duration::minutes(30)).unix_timestamp()) as i64;
        let claims = serde_json::json!( {
            "user": {
                "id": 1,
                "email": "test@test.com".to_string(),
            },
            "owner": {
                "id": 1,
                "email": "test@test.com"

            },
            "store_hash": "test-store",
            "exp": exp,
        });
        let header = Header::new(Algorithm::HS256);
        let key = EncodingKey::from_secret(self.bc_secret.expose_secret().as_bytes());

        encode(&header, &claims, &key).unwrap()
    }

    pub fn generate_local_jwt_token(&self) -> String {
        create_jwt("test-store", &self.jwt_secret).unwrap()
    }
}
