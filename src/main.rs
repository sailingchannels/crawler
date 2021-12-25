use mongodb::{options::ClientOptions, Client};
use repos::additional_channel_repo::AdditionalChannelRepository;
use std::env;
use tokio::time::{sleep, Duration};

mod repos;

#[tokio::main]
pub async fn main() -> Result<(), anyhow::Error> {
    let fifteen_minutes_in_seconds: u64 = 15 * 60;

    let mongodb_default_conn = "mongodb://localhost:27017".to_string();
    let connection_string = env::var("MONGO_CONNECTION_STRING").unwrap_or(mongodb_default_conn);

    let opts = ClientOptions::parse(connection_string).await?;
    let client = Client::with_options(opts)?;

    loop {
        let additional_channel_repo = AdditionalChannelRepository::new(&client);
        let additional_channels = additional_channel_repo.get_all().await?;

        println!("{:?}", additional_channels.len());

        println!("Wait 15 minutes till next execution...");
        sleep(Duration::from_secs(fifteen_minutes_in_seconds)).await;
    }
}
