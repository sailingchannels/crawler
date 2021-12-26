use crawler::additional_channel_crawler::AdditionalChannelCrawler;
use log::{info, LevelFilter};
use mongodb::{options::ClientOptions, Client};
use repos::additional_channel_repo::AdditionalChannelRepository;
use simple_logger::SimpleLogger;
use std::env;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task;

mod crawler;
mod repos;
mod scraper;
mod services;

#[tokio::main]
pub async fn main() -> Result<(), anyhow::Error> {
    SimpleLogger::new().with_level(LevelFilter::Info).init()?;

    let mongodb_default_conn = "mongodb://localhost:27017".to_string();
    let connection_string = env::var("MONGO_CONNECTION_STRING").unwrap_or(mongodb_default_conn);

    info!("Start connection to mongodb");

    let opts = ClientOptions::parse(connection_string).await?;
    let client = Client::with_options(opts)?;

    info!("Connected to mongodb");

    let mut tasks = vec![];

    let (tx, mut rx): (Sender<String>, Receiver<String>) = channel(32);

    let additional_channel_crawling_task = task::spawn(async move {
        let additional_channel_repo = AdditionalChannelRepository::new(&client);
        let additional_channel_crawler = AdditionalChannelCrawler::new(tx, additional_channel_repo);

        info!("Start additional channel crawling");
        additional_channel_crawler.crawl().await.unwrap();
    });

    tasks.push(additional_channel_crawling_task);

    let channel_scraper_task = task::spawn(async move {
        info!("Start channel scrape listener");

        // Start receiving messages
        while let Some(channel_id) = rx.recv().await {
            info!("Received channel id: {}", channel_id);
        }
    });

    tasks.push(channel_scraper_task);

    for task in tasks {
        task.await.expect("Panic in task");
    }

    Ok(())
}
