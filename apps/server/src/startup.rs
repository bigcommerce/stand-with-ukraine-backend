use crate::configuration::{Configuration, Database};
use crate::routes;
use crate::state::Shared;
use axum::serve::Serve;
use axum::Router;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::TcpListener;

pub struct Application {
    port: u16,
    server: Serve<Router, Router>,
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
        let server = run(listener, configuration.get_app_state());

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

#[allow(clippy::default_constructed_unit_structs)]
// reason = "OtelInResponseLayer struct is external and might change"
pub fn run(listener: TcpListener, shared_state: Shared) -> Serve<Router, Router> {
    let app = Router::new()
        .merge(routes::router())
        .layer(OtelInResponseLayer::default())
        .layer(OtelAxumLayer::default())
        .with_state(shared_state);

    axum::serve(listener, app)
}
