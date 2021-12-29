use anyhow::Error;
use tokio::sync::mpsc::Sender;

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
        let channels = self.channel_repo.get_all_ids().await?;

        for channel in channels {
            let command = CrawlVideosCommand {
                channel_id: channel.clone(),
            };

            self.sender.send(command).await?;
        }

        Ok(())
    }
}
