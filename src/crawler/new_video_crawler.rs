use anyhow::Error;
use log::info;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

const FIVE_MINUTES_IN_SECONDS: u64 = 5 * 60;

use crate::{
    commands::crawl_videos_command::CrawlVideosCommand, repos::channel_repo::ChannelRepository,
};

pub struct NewVideoCrawler {
    sender: Sender<CrawlVideosCommand>,
    channel_repo: ChannelRepository,
}

impl NewVideoCrawler {
    pub fn new(
        sender: Sender<CrawlVideosCommand>,
        channel_repo: ChannelRepository,
    ) -> NewVideoCrawler {
        NewVideoCrawler {
            sender,
            channel_repo,
        }
    }

    pub async fn crawl(&self) -> Result<(), Error> {
        loop {
            info!("Start new video crawler");

            let channels = self.channel_repo.get_all_ids().await?;

            for channel in channels {
                let command = CrawlVideosCommand {
                    channel_id: channel.clone(),
                };

                self.sender.send(command).await?;
            }

            info!(
                "Wait for {} seconds until next crawl",
                FIVE_MINUTES_IN_SECONDS
            );

            sleep(Duration::from_secs(FIVE_MINUTES_IN_SECONDS)).await;
        }
    }
}
