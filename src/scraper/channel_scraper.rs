use anyhow::Error;
use chrono::{Duration, Utc};
use log::{debug, info};

use crate::repos::channel_repo::ChannelRepository;

pub struct ChannelScraper {
    channel_repo: ChannelRepository,
}

impl ChannelScraper {
    pub fn new(channel_repo: ChannelRepository) -> ChannelScraper {
        ChannelScraper { channel_repo }
    }

    pub async fn scrape(
        &self,
        channel_id: String,
        ignore_sailing_terms: bool,
    ) -> Result<(), Error> {
        let should_crawl = self.should_crawl_now(&channel_id).await;
        if should_crawl == false {
            info!(
                "Channel {} is not crawled now, last crawl less then a day ago",
                channel_id
            );

            return Ok(());
        }

        info!("Start scraping channel {}", channel_id);

        Ok(())
    }

    async fn should_crawl_now(&self, channel_id: &str) -> bool {
        let one_day_in_seconds = 86400;

        let last_crawl_default = Utc::now() - Duration::seconds(one_day_in_seconds + 1);

        let last_crawl = self
            .channel_repo
            .get_last_crawl_date(channel_id)
            .await
            .unwrap_or(mongodb::bson::DateTime::from_millis(
                last_crawl_default.timestamp_millis(),
            ));

        let current_timestamp = Utc::now().timestamp();
        let should_crawl =
            current_timestamp - (last_crawl.timestamp_millis() / 1000) < one_day_in_seconds;

        should_crawl
    }
}
