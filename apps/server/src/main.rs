use swu_app::{configuration::Configuration, startup::Application, telemetry::init_tracing};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_tracing("swu-app".into(), "info".into());

    let configuration =
        Configuration::generate_from_environment().expect("Failed to read configuration.");
    let application = Application::build(configuration)?;

    application.run_until_stopped().await?;

    Ok(())
}
