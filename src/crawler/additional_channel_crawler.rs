use anyhow::Error;
use log::info;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

use crate::commands::crawl_channel_command::CrawlChannelCommand;
use crate::repos::additional_channel_repo::AdditionalChannelRepository;

const TEN_MINUTES_IN_SECONDS: u64 = 10 * 60;

pub struct AdditionalChannelCrawler {
    sender: Sender<CrawlChannelCommand>,
    additional_channel_repo: AdditionalChannelRepository,
}

impl AdditionalChannelCrawler {
    pub fn new(
        sender: Sender<CrawlChannelCommand>,
        additional_channel_repo: AdditionalChannelRepository,
    ) -> AdditionalChannelCrawler {
        AdditionalChannelCrawler {
            sender,
            additional_channel_repo,
        }
    }

    pub async fn crawl(&self) -> Result<(), Error> {
        loop {
            info!("Start additional channel crawler");
            let additional_channels = self.additional_channel_repo.get_all().await?;

            info!("Found {} additional channels", additional_channels.len());

            for additional_channel in additional_channels {
                let channel_id = additional_channel.get_str("_id")?.to_string();
                let ignore_sailing_terms = additional_channel.get_bool("ignoreSailingTerm")?;

                info!("Send additional channel for crawling: {}", channel_id);

                let cmd = CrawlChannelCommand {
                    channel_id,
                    ignore_sailing_terms,
                };

                self.sender.send(cmd).await?;
            }

            info!(
                "Wait for {} seconds until next crawl",
                TEN_MINUTES_IN_SECONDS
            );

            sleep(Duration::from_secs(TEN_MINUTES_IN_SECONDS)).await;
        }
    }
}
