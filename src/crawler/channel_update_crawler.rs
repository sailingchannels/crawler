use anyhow::Error;
use chrono::Utc;
use log::info;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

use crate::{
    commands::crawl_channel_command::CrawlChannelCommand, repos::channel_repo::ChannelRepository,
};

const FIFTEEN_MINUTES_IN_SECONDS: u64 = 15 * 60;

pub struct ChannelUpdateCrawler {
    channel_repo: ChannelRepository,
    sender: Sender<CrawlChannelCommand>,
}

impl ChannelUpdateCrawler {
    pub fn new(
        sender: Sender<CrawlChannelCommand>,
        channel_repo: ChannelRepository,
    ) -> ChannelUpdateCrawler {
        ChannelUpdateCrawler {
            channel_repo,
            sender,
        }
    }

    pub async fn crawl(&self) -> Result<(), Error> {
        loop {
            info!("Start channel update crawler");

            let last_crawl_before = Utc::now() - chrono::Duration::days(1);
            let last_upload_after = Utc::now() - chrono::Duration::weeks(52);
            let channel_ids = self
                .channel_repo
                .get_ids_last_crawled_before(last_crawl_before, last_upload_after)
                .await?;

            info!("Found {} channels to update", channel_ids.len());

            for channel_id in channel_ids {
                let cmd = CrawlChannelCommand {
                    channel_id,
                    ignore_sailing_terms: false,
                };

                self.sender.send(cmd).await?;
            }

            info!(
                "Wait for {} seconds until next crawl",
                FIFTEEN_MINUTES_IN_SECONDS
            );

            sleep(Duration::from_secs(FIFTEEN_MINUTES_IN_SECONDS)).await;
        }
    }
}
