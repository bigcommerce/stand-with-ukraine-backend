use configuration::Configuration;

mod configuration;
mod sheets;
mod startup;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration =
        Configuration::generate_from_environment().expect("Failed to read configuration.");

    startup::run(configuration).await;

    Ok(())
}
