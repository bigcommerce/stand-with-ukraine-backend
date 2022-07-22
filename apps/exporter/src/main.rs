use configuration::Configuration;
use dotenv::dotenv;

mod configuration;
mod sheets;
mod startup;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let configuration =
        Configuration::generate_from_environment().expect("Failed to read configuration.");

    startup::run(configuration).await;

    Ok(())
}
