use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{web, Error};
use opentelemetry::{global, sdk::propagation::TraceContextPropagator};
use secrecy::ExposeSecret;
use tracing::{dispatcher::set_global_default, Span, Subscriber};
use tracing_actix_web::{root_span, DefaultRootSpanBuilder, RootSpanBuilder};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{
    fmt::MakeWriter, prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry,
};

use crate::configuration::LightstepAccessToken;

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Sync + Send
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name(&name)
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("Failed to install OpenTelemetry");
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    Registry::default()
        .with(env_filter)
        .with(telemetry)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Sync + Send) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber.into()).expect("Failed to set subscriber");
}

pub struct AppRootSpanBuilder;

impl RootSpanBuilder for AppRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        let access_token = match request.app_data::<web::Data<LightstepAccessToken>>() {
            // unwrap from web::Data => LightstepAccessToken => Secret => String => str
            Some(access_token) => access_token.as_ref().as_ref().expose_secret().as_str(),
            None => "developer",
        };

        root_span!(request, lightstep.access_token = access_token)
    }

    fn on_request_end<B>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}
