// https://github.com/sailingchannels/crawler/blob/76b4442032e9062537576e98e37180c01293b412/discovery.py

use crate::{
    commands::crawl_channel_command::CrawlChannelCommand,
    repos::{
        additional_channel_repo::AdditionalChannelRepository, channel_repo::ChannelRepository,
        non_sailing_channel_repo::NonSailingChannelRepository, settings_repo::SettingsRepository,
    },
    services::{sailing_terms_service::SailingTermsService, youtube_service::YoutubeService},
};
use anyhow::Error;
use chrono::Utc;
use log::info;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

const ONE_DAYS_IN_SECONDS: u64 = 86400;

pub struct ChannelDiscoveryCrawler {
    sender: Sender<CrawlChannelCommand>,
    channel_repo: ChannelRepository,
    settings_repo: SettingsRepository,
    youtube_service: YoutubeService,
    sailing_terms_service: SailingTermsService,
    additional_channel_repo: AdditionalChannelRepository,
    non_sailing_channel_repo: NonSailingChannelRepository,
}

impl ChannelDiscoveryCrawler {
    pub fn new(
        sender: Sender<CrawlChannelCommand>,
        channel_repo: ChannelRepository,
        settings_repo: SettingsRepository,
        youtube_service: YoutubeService,
        sailing_terms_service: SailingTermsService,
        additional_channel_repo: AdditionalChannelRepository,
        non_sailing_channel_repo: NonSailingChannelRepository,
    ) -> ChannelDiscoveryCrawler {
        ChannelDiscoveryCrawler {
            sender,
            channel_repo,
            settings_repo,
            youtube_service,
            sailing_terms_service,
            additional_channel_repo,
            non_sailing_channel_repo,
        }
    }

    pub async fn crawl(&self) -> Result<(), Error> {
        println!("Start channel discovery crawler");

        loop {
            if self.should_crawl().await.unwrap_or(false) {
                let channel_ids = self.channel_repo.get_ids_upload_last_three_month().await?;

                for channel_id in channel_ids {
                    info!("Check subscriptions of channel {}", channel_id);

                    let subscriptions = self
                        .youtube_service
                        .get_channel_subscriptions(&channel_id)
                        .await?;

                    for snippet in subscriptions {
                        let sailing_terms_result = self
                            .sailing_terms_service
                            .has_sailing_term(
                                &snippet.channel_id,
                                &snippet.title,
                                &snippet.description,
                                false,
                            )
                            .await;

                        let is_newly_discovered = self
                            .is_channel_newly_discovered(&snippet.channel_id)
                            .await?;

                        let is_not_non_sailing_channel =
                            self.is_not_non_sailing_channel(&snippet.channel_id).await?;

                        if is_newly_discovered
                            && is_not_non_sailing_channel
                            && sailing_terms_result.has_sailing_term
                        {
                            info!("Send channel for crawling: {}", snippet.channel_id);

                            let cmd = CrawlChannelCommand {
                                channel_id: snippet.channel_id.clone(),
                                ignore_sailing_terms: false,
                            };

                            self.sender.send(cmd).await?;
                        }
                    }
                }
            }

            info!("Wait for {} seconds until next crawl", ONE_DAYS_IN_SECONDS);

            sleep(Duration::from_secs(ONE_DAYS_IN_SECONDS)).await;
        }
    }

    async fn should_crawl(&self) -> Result<bool, Error> {
        let last_crawl_timestamp = self.settings_repo.get_last_subscriber_crawl().await?;
        let seconds_since_last_crawl = Utc::now().timestamp() - last_crawl_timestamp;

        Ok(seconds_since_last_crawl >= ONE_DAYS_IN_SECONDS as i64)
    }

    async fn is_channel_newly_discovered(&self, channel_id: &str) -> Result<bool, Error> {
        let channel_exists = self.channel_repo.exists(channel_id).await?;
        let additional_exists = self.additional_channel_repo.exists(channel_id).await?;

        Ok(!channel_exists && !additional_exists)
    }

    async fn is_not_non_sailing_channel(&self, channel_id: &str) -> Result<bool, Error> {
        let non_sailing_channel_exists = self.non_sailing_channel_repo.exists(channel_id).await?;

        Ok(!non_sailing_channel_exists)
    }
}
