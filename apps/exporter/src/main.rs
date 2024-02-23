#![deny(unused_extern_crates)]

use configuration::Configuration;

mod configuration;
mod report;
mod sheets;
mod time;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration =
        Configuration::generate_from_environment().expect("Failed to read configuration.");

    report::run(configuration).await;

    Ok(())
}
