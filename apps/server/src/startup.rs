use crate::liq_pay::HttpAPI as LiqPayHttpAPI;
use crate::routes;
use crate::{
    bigcommerce::client::HttpAPI as BigCommerceHttpAPI,
    configuration::{Configuration, Database},
};
use axum::serve::Serve;
use axum::{Extension, Router};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use secrecy::Secret;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::TcpListener;

pub struct Application {
    port: u16,
    server: Serve<Router, Router>,
}

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub base_url: String,
    pub jwt_secret: Secret<String>,
    pub bigcommerce_client: BigCommerceHttpAPI,
    pub liq_pay_client: LiqPayHttpAPI,
}

impl Application {
    /// # Errors
    ///
    /// Will return `std::io::Error` if listener could not be setup on the port provided
    pub async fn build(configuration: Configuration) -> Result<Self, std::io::Error> {
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address).await?;
        let port = listener
            .local_addr()
            .expect("listener does not have an address")
            .port();
        let server = run(listener, configuration.get_app_state())?;

        Ok(Self { port, server })
    }

    pub const fn port(&self) -> u16 {
        self.port
    }

    /// # Errors
    ///
    /// Will return `std::io::Error` if actix server returns an error
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

#[must_use]
pub fn get_connection_pool(configuration: &Database) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

/// # Errors
///
/// Will return `std::io::Error` if server could not bind to the listener
pub fn run(
    listener: TcpListener,
    state: AppState,
) -> Result<Serve<Router, Router>, std::io::Error> {
    let app = Router::new()
        .merge(routes::router())
        .layer(OtelInResponseLayer::default())
        .layer(OtelAxumLayer::default())
        .with_state(state.clone())
        .layer(Extension(state));

    let server = axum::serve(listener, app);

    Ok(server)
}
