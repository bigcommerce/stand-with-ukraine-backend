use std::net::TcpListener;

use crate::{
    bigcommerce::client::HttpAPI,
    configuration::{BaseURL, Configuration, Database, JWTSecret, LightstepAccessToken},
    routes::register,
    telemetry::AppRootSpanBuilder,
};
use actix_web::{dev::Server, web::Data, App, HttpServer};
use secrecy::Secret;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    /// # Errors
    ///
    /// Will return `std::io::Error` if listener could not be setup on the port provided
    pub fn build(configuration: Configuration) -> Result<Self, std::io::Error> {
        let db_pool = get_connection_pool(&configuration.database);
        let bigcommerce_client = HttpAPI::new(
            configuration.bigcommerce.api_base_url,
            configuration.bigcommerce.login_base_url,
            configuration.bigcommerce.client_id,
            configuration.bigcommerce.client_secret,
            configuration.bigcommerce.install_redirect_uri,
            std::time::Duration::from_millis(configuration.bigcommerce.timeout.into()),
        );

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener
            .local_addr()
            .expect("listener does not have an address")
            .port();
        let server = run(
            listener,
            db_pool,
            configuration.application.base_url,
            configuration.application.jwt_secret,
            configuration.application.lightstep_access_token,
            bigcommerce_client,
        )?;

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
    db_pool: PgPool,
    base_url: String,
    jwt_secret: Secret<String>,
    lightstep_access_token: Secret<String>,
    bigcommerce_client: HttpAPI,
) -> Result<Server, std::io::Error> {
    let db_pool = Data::new(db_pool);
    let base_url = Data::new(BaseURL(base_url));
    let bigcommerce_client = Data::new(bigcommerce_client);
    let jwt_secret = Data::new(JWTSecret(jwt_secret));
    let lightstep_access_token = Data::new(LightstepAccessToken(lightstep_access_token));

    let server = HttpServer::new(move || {
        App::new()
            .app_data(db_pool.clone())
            .app_data(base_url.clone())
            .app_data(bigcommerce_client.clone())
            .app_data(jwt_secret.clone())
            .app_data(lightstep_access_token.clone())
            .wrap(TracingLogger::<AppRootSpanBuilder>::new())
            .configure(register)
    })
    .listen(listener)?
    .run();

    Ok(server)
}
