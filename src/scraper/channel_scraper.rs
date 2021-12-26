use anyhow::Error;
use log::info;

use crate::repos::channel_repo::ChannelRepository;

pub struct ChannelScraper {
    channel_repo: ChannelRepository,
}

impl ChannelScraper {
    pub fn new(channel_repo: ChannelRepository) -> ChannelScraper {
        ChannelScraper { channel_repo }
    }

    pub async fn scrape(&self, channel_id: String) -> Result<(), Error> {
        info!("Start scraping channel {}", channel_id);
        Ok(())
    }
}
