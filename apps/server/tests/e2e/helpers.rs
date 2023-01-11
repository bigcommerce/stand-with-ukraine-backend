use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use once_cell::sync::Lazy;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use swu_app::{
    authentication::create_jwt,
    bigcommerce::auth::BCUser,
    configuration::{Configuration, Database, JWTSecret},
    data::WidgetConfiguration,
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
    pub bc_redirect_uri: String,

    pub test_client: Client,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let bigcommerce_server = MockServer::start().await;

    // configuration for this test instance
    let configuration = {
        let mut c =
            Configuration::generate_from_environment().expect("Failed to read configuration.");
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

    TestApp {
        address: format!("http://127.0.0.1:{}", application_port),
        port: application_port,
        bigcommerce_server,
        db_pool: get_connection_pool(&configuration.database),
        jwt_secret: JWTSecret(configuration.application.jwt_secret),
        bc_secret: configuration.bigcommerce.client_secret,
        bc_redirect_uri: configuration.bigcommerce.install_redirect_uri,
        base_url: configuration.application.base_url,
        test_client: reqwest::Client::new(),
    }
}

async fn configure_database(config: &Database) -> PgPool {
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
    sqlx::migrate!("../../migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}

impl TestApp {
    pub fn generate_bc_jwt_token(&self) -> String {
        let user = BCUser {
            id: 1,
            email: "test@test.com".to_owned(),
        };

        self.generate_bc_jwt_token_with_params("store/test-store", &user, &user)
    }

    pub fn generate_bc_jwt_token_with_params(
        &self,
        sub: &str,
        owner: &BCUser,
        user: &BCUser,
    ) -> String {
        let now = OffsetDateTime::now_utc();
        let expiration = now + Duration::minutes(30);
        let claims = serde_json::json!( {
            "iss": "bc",
            "iat": now.unix_timestamp(),
            "exp": expiration.unix_timestamp(),
            "sub": sub,
            "user": user,
            "owner": owner,
            "url": "/",
            "channel_id": null
        });
        let header = Header::new(Algorithm::HS256);
        let key = EncodingKey::from_secret(self.bc_secret.expose_secret().as_bytes());

        encode(&header, &claims, &key).unwrap()
    }

    pub fn generate_local_jwt_token(&self) -> String {
        create_jwt("test-store", &self.jwt_secret).unwrap()
    }

    pub async fn insert_test_store(&self) {
        sqlx::query!(
            r#"
            INSERT INTO stores (id, store_hash, access_token, installed_at, uninstalled)
            VALUES (gen_random_uuid(), 'test-store', 'test-token', '2021-04-20 00:00:00-07'::timestamptz, false)
            "#,
        )
        .execute(&self.db_pool)
        .await
        .unwrap();
    }

    pub fn test_server_url(&self, path: &str) -> String {
        format!("{}{}", &self.address, path)
    }

    pub async fn get_widget_events(&self, store_hash: &str) -> impl Iterator<Item = String> {
        sqlx::query!(
            "SELECT event_type FROM widget_events WHERE store_hash = $1;",
            store_hash
        )
        .fetch_all(&self.db_pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.event_type)
    }

    pub async fn get_form_feedback_submissions(
        &self,
    ) -> impl Iterator<Item = (String, String, String)> {
        sqlx::query!("SELECT name, email, message FROM feedback_form;")
            .fetch_all(&self.db_pool)
            .await
            .unwrap()
            .into_iter()
            .map(|row| (row.name, row.email, row.message))
    }

    pub async fn get_charity_visited_events(
        &self,
        store_hash: &str,
    ) -> impl Iterator<Item = (String, String)> {
        sqlx::query!(
            "SELECT charity, event_type FROM charity_events WHERE store_hash = $1;",
            store_hash
        )
        .fetch_all(&self.db_pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| (row.charity, row.event_type))
    }
}

pub fn create_test_server_client_no_redirect() -> Client {
    reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

pub fn get_widget_configuration() -> WidgetConfiguration {
    WidgetConfiguration {
        style: "blue".to_owned(),
        placement: "top-left".to_owned(),
        charity_selections: vec!["razom".to_owned()],
        modal_title: "Title!".to_owned(),
        modal_body: "Body!".to_owned(),
    }
}
