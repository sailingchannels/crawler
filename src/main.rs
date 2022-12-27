use std::str::FromStr;

use crawler::{
    additional_channel_crawler::AdditionalChannelCrawler,
    channel_discovery_crawler::ChannelDiscoveryCrawler,
};
use figment::{
    providers::{Env, Format, Json},
    Figment,
};
use log::{debug, error, info, LevelFilter};
use mongodb::{options::ClientOptions, Client};
use repos::additional_channel_repo::AdditionalChannelRepository;
use repos::blacklist_repo::BlacklistRepository;
use repos::sailing_term_repo::SailingTermRepository;
use simple_logger::SimpleLogger;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::{self, JoinHandle};

use crate::{
    commands::crawl_channel_command::CrawlChannelCommand,
    repos::{
        settings_repo::SettingsRepository, subscriber_repo::SubscriberRepository,
        view_repo::ViewRepository,
    },
    services::{sailing_terms_service::SailingTermsService, youtube_service::YoutubeService},
};
use crate::{commands::crawl_videos_command::CrawlVideosCommand, models::config::Config};
use crate::{
    crawler::channel_update_crawler::ChannelUpdateCrawler, repos::video_repo::VideoRepository,
};
use crate::{crawler::new_video_crawler::NewVideoCrawler, repos::channel_repo::ChannelRepository};
use crate::{repos::apikeys_repo::ApiKeyRepository, scraper::channel_scraper::ChannelScraper};
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
    info!("Environment {}", config.environment);

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

    register_additional_channel_crawler(
        &mut tasks,
        db_client.clone(),
        config.clone(),
        channel_scraper_tx.clone(),
    );

    register_channel_discovery_crawler(
        &mut tasks,
        db_client.clone(),
        config.clone(),
        channel_scraper_tx.clone(),
    );

    register_channel_update_crawler(
        &mut tasks,
        db_client.clone(),
        config.clone(),
        channel_scraper_tx.clone(),
    );

    register_new_video_crawler(
        &mut tasks,
        db_client.clone(),
        config.clone(),
        video_scraper_tx.clone(),
    );

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
    config: Config,
    tx: Sender<CrawlChannelCommand>,
) {
    if config.crawler.additional == false {
        return;
    }

    let additional_channel_crawling_task = task::spawn(async move {
        let additional_channel_repo =
            AdditionalChannelRepository::new(&mongo_client, &config.environment);
        let crawler = AdditionalChannelCrawler::new(tx, additional_channel_repo);

        info!("CRAWLER: Start additional channel crawling");
        crawler
            .crawl()
            .await
            .expect("Panic in additional channel crawling");
    });

    tasks.push(additional_channel_crawling_task);
}

fn register_channel_discovery_crawler(
    tasks: &mut Vec<JoinHandle<()>>,
    mongo_client: Client,
    config: Config,
    tx: Sender<CrawlChannelCommand>,
) {
    if config.crawler.discovery == false {
        return;
    }

    let channel_discovery_crawling_task = task::spawn(async move {
        let sailing_terms = get_sailing_terms(&mongo_client, &config.environment).await;
        let blacklisted_channel_ids =
            get_blacklisted_channels(&mongo_client, &config.environment).await;

        let channel_repo = ChannelRepository::new(&mongo_client, &config.environment);
        let settings_repo = SettingsRepository::new(&mongo_client, &config.environment);
        let apikey_repo = ApiKeyRepository::new(&mongo_client, &config.environment);
        let non_sailing_channel_repo =
            NonSailingChannelRepository::new(&mongo_client, &config.environment);
        let additional_channel_repo =
            AdditionalChannelRepository::new(&mongo_client, &config.environment);

        let youtube_service = YoutubeService::new(apikey_repo);
        let sailing_terms_service = SailingTermsService::new(
            sailing_terms,
            blacklisted_channel_ids,
            non_sailing_channel_repo,
        );

        let crawler = ChannelDiscoveryCrawler::new(
            tx,
            channel_repo,
            settings_repo,
            youtube_service,
            sailing_terms_service,
            additional_channel_repo,
        );

        info!("CRAWLER: Start channel discovery crawling");
        crawler
            .crawl()
            .await
            .expect("Panic in channel discovery crawling");
    });

    tasks.push(channel_discovery_crawling_task);
}

fn register_channel_update_crawler(
    tasks: &mut Vec<JoinHandle<()>>,
    mongo_client: Client,
    config: Config,
    tx: Sender<CrawlChannelCommand>,
) {
    if config.crawler.channel == false {
        return;
    }

    let channel_update_crawling_task = task::spawn(async move {
        let channel_repo = ChannelRepository::new(&mongo_client, &config.environment);
        let crawler = ChannelUpdateCrawler::new(tx, channel_repo);

        info!("CRAWLER: Start channel update crawling");
        let result = crawler.crawl().await;

        if let Err(e) = result {
            error!("Error in channel update crawling: {}", e);
        }
    });

    tasks.push(channel_update_crawling_task);
}

fn register_new_video_crawler(
    tasks: &mut Vec<JoinHandle<()>>,
    mongo_client: Client,
    config: Config,
    tx: Sender<CrawlVideosCommand>,
) {
    if config.crawler.video == false {
        return;
    }

    let new_video_crawling_task = task::spawn(async move {
        let channel_repo = ChannelRepository::new(&mongo_client, &config.environment);
        let crawler = NewVideoCrawler::new(tx, channel_repo);

        info!("CRAWLER: Start new video crawling");
        let result = crawler.crawl().await;

        if let Err(e) = result {
            error!("Error in new video crawling: {}", e);
        }
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
        info!("SCRAPER: Start channel scrape listener");

        let channel_repo = ChannelRepository::new(&mongo_client, &config.environment);
        let non_sailing_channel_repo =
            NonSailingChannelRepository::new(&mongo_client, &config.environment);
        let view_repo = ViewRepository::new(&mongo_client, &config.environment);
        let subscriber_repo = SubscriberRepository::new(&mongo_client, &config.environment);
        let video_repo = VideoRepository::new(&mongo_client, &config.environment);
        let apikey_repo = ApiKeyRepository::new(&mongo_client, &config.environment);

        let sailing_terms = get_sailing_terms(&mongo_client, &config.environment).await;
        let blacklisted_channel_ids =
            get_blacklisted_channels(&mongo_client, &config.environment).await;

        let sailing_terms_service = SailingTermsService::new(
            sailing_terms,
            blacklisted_channel_ids,
            non_sailing_channel_repo,
        );

        let scraper = ChannelScraper::new(
            channel_repo,
            view_repo,
            subscriber_repo,
            video_repo,
            apikey_repo,
            sailing_terms_service,
        );

        while let Some(cmd) = rx.recv().await {
            let result = scraper
                .scrape(cmd.channel_id, cmd.ignore_sailing_terms)
                .await;

            if let Err(e) = result {
                error!("Error in channel scraping: {}", e);
            }
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
        info!("SCRAPER: Start video scrape listener");

        let video_repo = VideoRepository::new(&mongo_client, &config.environment);
        let channel_repo = ChannelRepository::new(&mongo_client, &config.environment);
        let scraper = VideoScraper::new(video_repo, channel_repo);

        while let Some(cmd) = rx.recv().await {
            let result = scraper.scrape(cmd.channel_id).await;

            if let Err(e) = result {
                error!("Error in video scraper: {}", e);
            }
        }
    });

    tasks.push(video_scraper_task);
}

async fn get_sailing_terms(mongo_client: &Client, environment: &str) -> Vec<String> {
    let sailing_term_repo = SailingTermRepository::new(&mongo_client, environment);
    let sailing_terms = sailing_term_repo.get_all().await.unwrap();

    sailing_terms
}

async fn get_blacklisted_channels(mongo_client: &Client, environment: &str) -> Vec<String> {
    let blacklist_repo = BlacklistRepository::new(&mongo_client, environment);
    let blacklisted_channels = blacklist_repo.get_all().await.unwrap();

    blacklisted_channels
}
