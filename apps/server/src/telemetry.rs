use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{web, Error};
use opentelemetry::{global, sdk::propagation::TraceContextPropagator};
use secrecy::ExposeSecret;
use tracing::Dispatch;
use tracing::{dispatcher::set_global_default, Span};
use tracing_actix_web::{root_span, DefaultRootSpanBuilder, RootSpanBuilder};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry};

use crate::configuration::LightstepAccessToken;

pub fn init_tracing(name: String, default_env_filter: String) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_env_filter));

    let dispatcher: Dispatch = if std::env::var("OTEL_ENABLE").is_ok() {
        global::set_text_map_propagator(TraceContextPropagator::new());

        let tracer = opentelemetry_jaeger::new_collector_pipeline()
            .with_endpoint("http://localhost:14268/api/traces")
            .with_service_name(&name)
            .with_reqwest()
            .install_batch(opentelemetry::runtime::Tokio)
            .expect("Failed to install OpenTelemetry");
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

        Registry::default().with(env_filter).with(telemetry).into()
    } else {
        let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);

        Registry::default()
            .with(env_filter)
            .with(JsonStorageLayer)
            .with(formatting_layer)
            .into()
    };

    set_global_default(dispatcher).expect("failed to set dispatcher");
}

pub struct AppRootSpanBuilder;

impl RootSpanBuilder for AppRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        let access_token = request
            .app_data::<web::Data<LightstepAccessToken>>()
            .map_or("developer", |access_token| {
                access_token.as_ref().as_ref().expose_secret().as_str()
            });

        root_span!(request, lightstep.access_token = access_token)
    }

    fn on_request_end<B>(span: Span, outcome: &Result<ServiceResponse<B>, Error>)
    where
        B: MessageBody,
    {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}
