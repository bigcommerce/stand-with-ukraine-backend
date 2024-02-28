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
        generate_otlp_tracing_subscriber(name, env_filter)
    } else {
        generate_bunyan_console_subscriber(name, env_filter)
    };

    set_global_default(dispatcher).expect("failed to set dispatcher");
}

pub fn generate_otlp_tracing_subscriber(name: String, env_filter: EnvFilter) -> Dispatch {
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
}

pub fn generate_bunyan_console_subscriber(name: String, env_filter: EnvFilter) -> Dispatch {
    let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
        .into()
}

#[cfg(test)]
mod test {
    use std::any::{Any, TypeId};

    use tracing_subscriber::EnvFilter;

    use super::*;

    #[tokio::test]
    async fn test_generate_otlp_subscriber() {
        let env_filter = EnvFilter::new("trace");

        let dispatch = generate_otlp_tracing_subscriber("swu-app".to_owned(), env_filter);

        assert_eq!(dispatch.type_id(), TypeId::of::<Dispatch>())
    }

    #[tokio::test]
    async fn test_generate_bunyan_subscriber() {
        let env_filter = EnvFilter::new("trace");

        let dispatch = generate_bunyan_console_subscriber("swu-app".to_owned(), env_filter);

        assert_eq!(dispatch.type_id(), TypeId::of::<Dispatch>())
    }
}
