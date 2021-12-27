use crawler::additional_channel_crawler::AdditionalChannelCrawler;
use log::{info, LevelFilter};
use mongodb::{options::ClientOptions, Client};
use repos::additional_channel_repo::AdditionalChannelRepository;
use repos::blacklist_repo::BlacklistRepository;
use repos::sailing_term_repo::SailingTermRepository;
use simple_logger::SimpleLogger;
use std::env;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::{self, JoinHandle};

use crate::commands::crawl_channel_command::CrawlChannelCommand;
use crate::crawler::channel_update_crawler::ChannelUpdateCrawler;
use crate::repos::channel_repo::ChannelRepository;
use crate::repos::non_sailing_channel_repo::NonSailingChannelRepository;
use crate::scraper::channel_scraper::ChannelScraper;

mod commands;
mod crawler;
mod models;
mod repos;
mod scraper;
mod services;
mod utils;

#[tokio::main]
pub async fn main() -> Result<(), anyhow::Error> {
    SimpleLogger::new().with_level(LevelFilter::Info).init()?;

    let mongodb_default_conn = "mongodb://localhost:27017".to_string();
    let connection_string = env::var("MONGO_CONNECTION_STRING").unwrap_or(mongodb_default_conn);

    info!("Start connection to mongodb");

    let opts = ClientOptions::parse(connection_string).await?;
    let db_client = Client::with_options(opts)?;

    info!("Connected to mongodb");

    let mut tasks = vec![];

    let (channel_scraper_tx, channel_scraper_rx) = channel::<CrawlChannelCommand>(usize::MAX >> 3);

    register_additional_channel_crawler(&mut tasks, db_client.clone(), channel_scraper_tx.clone());
    register_channel_update_crawler(&mut tasks, db_client.clone(), channel_scraper_tx.clone());

    register_channel_scraper(&mut tasks, db_client.clone(), channel_scraper_rx);

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
    tx: Sender<CrawlChannelCommand>,
) {
    let additional_channel_crawling_task = task::spawn(async move {
        let additional_channel_repo = AdditionalChannelRepository::new(&mongo_client);
        let crawler = AdditionalChannelCrawler::new(tx, additional_channel_repo);

        info!("Start additional channel crawling");
        crawler.crawl().await.unwrap();
    });

    tasks.push(additional_channel_crawling_task);
}

fn register_channel_update_crawler(
    tasks: &mut Vec<JoinHandle<()>>,
    mongo_client: Client,
    tx: Sender<CrawlChannelCommand>,
) {
    let channel_update_crawling_task = task::spawn(async move {
        let channel_repo = ChannelRepository::new(&mongo_client);
        let crawler = ChannelUpdateCrawler::new(tx, channel_repo);

        info!("Start channel update crawling");
        crawler.crawl().await.unwrap();
    });

    tasks.push(channel_update_crawling_task);
}

fn register_channel_scraper(
    tasks: &mut Vec<JoinHandle<()>>,
    mongo_client: Client,
    mut rx: Receiver<CrawlChannelCommand>,
) {
    let channel_scraper_task = task::spawn(async move {
        info!("Start channel scrape listener");

        let channel_repo = ChannelRepository::new(&mongo_client);
        let non_sailing_channel_repo = NonSailingChannelRepository::new(&mongo_client);

        let sailing_terms = get_sailing_terms(&mongo_client).await;
        let blacklisted_channel_ids = get_blacklisted_channels(&mongo_client).await;

        let scraper = ChannelScraper::new(
            channel_repo,
            non_sailing_channel_repo,
            sailing_terms,
            blacklisted_channel_ids,
        );

        while let Some(cmd) = rx.recv().await {
            scraper
                .scrape(cmd.channel_id, cmd.ignore_sailing_terms)
                .await
                .unwrap();
        }
    });

    tasks.push(channel_scraper_task);
}

async fn get_sailing_terms(mongo_client: &Client) -> Vec<String> {
    let sailing_term_repo = SailingTermRepository::new(&mongo_client);
    let sailing_terms = sailing_term_repo.get_all().await.unwrap();

    sailing_terms
}

async fn get_blacklisted_channels(mongo_client: &Client) -> Vec<String> {
    let blacklist_repo = BlacklistRepository::new(&mongo_client);
    let blacklisted_channels = blacklist_repo.get_all().await.unwrap();

    blacklisted_channels
}
