use anyhow::Error;
use log::info;

use crate::repos::channel_repo::ChannelRepository;

pub struct ChannelUpdateCrawler {
    channel_repo: ChannelRepository,
}

impl ChannelUpdateCrawler {
    pub fn new(channel_repo: ChannelRepository) -> ChannelUpdateCrawler {
        ChannelUpdateCrawler { channel_repo }
    }

    pub async fn crawl(&self, channel_id: String) -> Result<(), Error> {
        info!("Start crawling channel {}", channel_id);
        Ok(())
    }
}
