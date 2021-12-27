use anyhow::Error;
use log::info;
use tokio::sync::mpsc::Sender;

use crate::{
    commands::crawl_channel_command::CrawlChannelCommand, repos::channel_repo::ChannelRepository,
};

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
        info!("Start channel update crawler");

        let channel_ids = self.channel_repo.get_all_ids().await?;

        for channel_id in channel_ids {
            let cmd = CrawlChannelCommand {
                channel_id,
                ignore_sailing_terms: false,
            };

            self.sender.send(cmd).await?;
        }

        Ok(())
    }
}
