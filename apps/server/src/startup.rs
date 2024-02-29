use crate::configuration::{Configuration, Database};
use crate::routes;
use crate::state::SharedState;
use axum::serve::Serve;
use axum::Router;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::TcpListener;
use tokio::signal;

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
        let server = build_server(listener, configuration.get_app_state());

        Ok(Self { port, server })
    }

    pub const fn port(&self) -> u16 {
        self.port
    }

    /// # Errors
    ///
    /// Will return `std::io::Error` if axum server returns an error
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.with_graceful_shutdown(shutdown_signal()).await
    }
}

#[must_use]
pub fn get_connection_pool(configuration: &Database) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

#[allow(clippy::default_constructed_unit_structs)]
// reason = "`OtelInResponseLayer` struct is an external dependency that might change"
pub fn build_server(listener: TcpListener, shared_state: SharedState) -> Serve<Router, Router> {
    let app = Router::new()
        .merge(routes::router())
        .layer(OtelInResponseLayer::default())
        .layer(OtelAxumLayer::default())
        .with_state(shared_state);

    axum::serve(listener, app)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {
        },
        () = terminate => {
        },
    }

    //TODO: remove when https://github.com/open-telemetry/opentelemetry-rust/issues/868 is fixed
    //for now we have to use async task because global tracer is in a RWLock that will block otherwise
    tokio::task::spawn_blocking(opentelemetry::global::shutdown_tracer_provider)
        .await
        .unwrap();
}
