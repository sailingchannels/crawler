use crawler::additional_channel_crawler::AdditionalChannelCrawler;
use log::{info, LevelFilter};
use mongodb::{options::ClientOptions, Client};
use repos::additional_channel_repo::AdditionalChannelRepository;
use simple_logger::SimpleLogger;
use std::env;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::{self, JoinHandle};

use crate::repos::channel_repo::ChannelRepository;
use crate::scraper::channel_scraper::ChannelScraper;

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
    let mongo_client = Client::with_options(opts)?;

    info!("Connected to mongodb");

    let mut tasks = vec![];

    let (channel_scraper_tx, channel_scraper_rx): (Sender<String>, Receiver<String>) =
        channel(usize::MAX >> 3);

    register_additional_channel_crawler(
        &mut tasks,
        mongo_client.clone(),
        channel_scraper_tx.clone(),
    );

    register_channel_scraper(&mut tasks, mongo_client.clone(), channel_scraper_rx);

    await_all(tasks).await;

    Ok(())
}

async fn await_all(tasks: Vec<JoinHandle<()>>) {
    for task in tasks {
        task.await.expect("Panic in task");
    }
}

fn register_additional_channel_crawler(
    tasks: &mut Vec<JoinHandle<()>>,
    mongo_client: Client,
    tx: Sender<String>,
) {
    let additional_channel_crawling_task = task::spawn(async move {
        let additional_channel_repo = AdditionalChannelRepository::new(&mongo_client);
        let additional_channel_crawler = AdditionalChannelCrawler::new(tx, additional_channel_repo);

        info!("Start additional channel crawling");
        additional_channel_crawler.crawl().await.unwrap();
    });

    tasks.push(additional_channel_crawling_task);
}

fn register_channel_scraper(
    tasks: &mut Vec<JoinHandle<()>>,
    mongo_client: Client,
    mut rx: Receiver<String>,
) {
    let channel_scraper_task = task::spawn(async move {
        info!("Start channel scrape listener");

        let channel_repo = ChannelRepository::new(&mongo_client);
        let channel_scraper = ChannelScraper::new(channel_repo);

        while let Some(channel_id) = rx.recv().await {
            channel_scraper.scrape(channel_id).await.unwrap();
        }
    });

    tasks.push(channel_scraper_task);
}
