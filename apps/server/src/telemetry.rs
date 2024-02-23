use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::runtime;
use opentelemetry_sdk::{
    trace::{self, Sampler},
    Resource,
};
use tracing::dispatcher::set_global_default;
use tracing::Dispatch;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry};

pub fn init_tracing(name: String, default_env_filter: String) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_env_filter));

    let dispatcher: Dispatch = if std::env::var("OTEL_ENABLE").is_ok() {
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint("http://localhost:4317"),
            )
            .with_trace_config(
                trace::config()
                    .with_sampler(Sampler::AlwaysOn)
                    .with_resource(Resource::new(vec![KeyValue::new("service.name", name)])),
            )
            .install_batch(runtime::Tokio)
            .expect("make tracing pipeline");

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
