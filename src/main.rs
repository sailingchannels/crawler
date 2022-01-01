use std::str::FromStr;

use crawler::additional_channel_crawler::AdditionalChannelCrawler;
use figment::{
    providers::{Env, Format, Json},
    Figment,
};
use log::{debug, info, LevelFilter};
use mongodb::{options::ClientOptions, Client};
use repos::additional_channel_repo::AdditionalChannelRepository;
use repos::blacklist_repo::BlacklistRepository;
use repos::sailing_term_repo::SailingTermRepository;
use simple_logger::SimpleLogger;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::{self, JoinHandle};

use crate::scraper::channel_scraper::ChannelScraper;
use crate::{
    commands::crawl_channel_command::CrawlChannelCommand,
    repos::{subscriber_repo::SubscriberRepository, view_repo::ViewRepository},
};
use crate::{commands::crawl_videos_command::CrawlVideosCommand, models::config::Config};
use crate::{
    crawler::channel_update_crawler::ChannelUpdateCrawler, repos::video_repo::VideoRepository,
};
use crate::{crawler::new_video_crawler::NewVideoCrawler, repos::channel_repo::ChannelRepository};
use crate::{
    repos::non_sailing_channel_repo::NonSailingChannelRepository,
    scraper::video_scraper::VideoScraper,
};

mod commands;
mod crawler;
mod models;
mod repos;
mod scraper;
mod services;
mod utils;

#[tokio::main]
pub async fn main() -> Result<(), anyhow::Error> {
    let config: Config = Figment::new()
        .merge(Json::file("config.json"))
        .merge(Env::raw().only(&["MONGO_CONNECTION_STRING"]))
        .extract()?;

    debug!("{:?}", config);

    SimpleLogger::new()
        .with_level(LevelFilter::from_str(&config.log_level).unwrap())
        .init()?;

    info!("Start connection to mongodb");

    let opts = ClientOptions::parse(&config.mongo_connection_string).await?;
    let db_client = Client::with_options(opts)?;

    info!("Connected to mongodb");

    let mut tasks = vec![];

    let (channel_scraper_tx, channel_scraper_rx) = channel::<CrawlChannelCommand>(usize::MAX >> 3);
    let (video_scraper_tx, video_scraper_rx) = channel::<CrawlVideosCommand>(usize::MAX >> 3);

    register_channel_scraper(
        &mut tasks,
        db_client.clone(),
        config.clone(),
        channel_scraper_rx,
    );

    register_video_scraper(
        &mut tasks,
        db_client.clone(),
        config.clone(),
        video_scraper_rx,
    );

    //register_additional_channel_crawler(&mut tasks, db_client.clone(), channel_scraper_tx.clone());
    //register_channel_update_crawler(&mut tasks, db_client.clone(), channel_scraper_tx.clone());
    register_new_video_crawler(&mut tasks, db_client.clone(), video_scraper_tx.clone());

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

fn register_new_video_crawler(
    tasks: &mut Vec<JoinHandle<()>>,
    mongo_client: Client,
    tx: Sender<CrawlVideosCommand>,
) {
    let new_video_crawling_task = task::spawn(async move {
        let channel_repo = ChannelRepository::new(&mongo_client);
        let crawler = NewVideoCrawler::new(tx, channel_repo);

        info!("Start new video crawling");
        crawler.crawl().await.unwrap();
    });

    tasks.push(new_video_crawling_task);
}

fn register_channel_scraper(
    tasks: &mut Vec<JoinHandle<()>>,
    mongo_client: Client,
    config: Config,
    mut rx: Receiver<CrawlChannelCommand>,
) {
    let channel_scraper_task = task::spawn(async move {
        info!("Start channel scrape listener");

        let channel_repo = ChannelRepository::new(&mongo_client);
        let non_sailing_channel_repo = NonSailingChannelRepository::new(&mongo_client);
        let view_repo = ViewRepository::new(&mongo_client);
        let subscriber_repo = SubscriberRepository::new(&mongo_client);
        let video_repo = VideoRepository::new(&mongo_client);

        let sailing_terms = get_sailing_terms(&mongo_client).await;
        let blacklisted_channel_ids = get_blacklisted_channels(&mongo_client).await;

        let scraper = ChannelScraper::new(
            channel_repo,
            view_repo,
            subscriber_repo,
            video_repo,
            non_sailing_channel_repo,
            sailing_terms,
            blacklisted_channel_ids,
            config.youtube_api_keys,
            config.environment,
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

fn register_video_scraper(
    tasks: &mut Vec<JoinHandle<()>>,
    mongo_client: Client,
    config: Config,
    mut rx: Receiver<CrawlVideosCommand>,
) {
    let video_scraper_task = task::spawn(async move {
        info!("Start video scrape listener");

        let video_repo = VideoRepository::new(&mongo_client);
        let channel_repo = ChannelRepository::new(&mongo_client);
        let scraper = VideoScraper::new(video_repo, channel_repo, config.youtube_api_keys, config.environment);

        while let Some(cmd) = rx.recv().await {
            scraper.scrape(cmd.channel_id).await.unwrap();
        }
    });

    tasks.push(video_scraper_task);
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
