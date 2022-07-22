use swu_app::{
    configuration::Configuration,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};
use dotenv::dotenv;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let subscriber = get_subscriber("swu-app".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration =
        Configuration::generate_from_environment().expect("Failed to read configuration.");
    let application = Application::build(configuration).await?;

    application.run_until_stopped().await?;

    Ok(())
}
