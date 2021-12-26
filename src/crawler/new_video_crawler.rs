use anyhow::Error;
use log::info;

use crate::repos::channel_repo::ChannelRepository;

pub struct NewVideoCrawler {
    channel_repo: ChannelRepository,
}

impl NewVideoCrawler {
    pub fn new(channel_repo: ChannelRepository) -> NewVideoCrawler {
        NewVideoCrawler { channel_repo }
    }

    pub async fn crawl(&self) -> Result<(), Error> {
        Ok(())
    }
}
