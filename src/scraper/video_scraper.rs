use anyhow::Error;
use chrono::{DateTime, FixedOffset};
use mongodb::bson::{doc, Document};
use quick_xml::de::from_str;

use crate::{
    models::youtube_video_feed_response::{Entry, YoutubeVideoFeedResponse},
    repos::{channel_repo::ChannelRepository, video_repo::VideoRepository},
};

const YOUTUBE_VIDEO_FEED_BASE_URL: &str = "https://www.youtube.com/feeds/videos.xml";

pub struct VideoScraper {
    video_repo: VideoRepository,
    channel_repo: ChannelRepository,
}

impl VideoScraper {
    pub fn new(video_repo: VideoRepository, channel_repo: ChannelRepository) -> Self {
        Self {
            video_repo,
            channel_repo,
        }
    }

    pub async fn scrape(&self, channel_id: String) -> Result<(), Error> {
        let channel_feed = load_and_parse_video_feed(&channel_id).await?;

        let mut videos = vec![];
        let mut max_last_upload_timestamp_millis: i64 = 0;

        for entry in channel_feed.entries.iter() {
            let published = DateTime::parse_from_rfc3339(&entry.published)?;

            let vid = self.build_video_document(&channel_id, &entry, published);
            videos.push(vid);

            if published.timestamp_millis() > max_last_upload_timestamp_millis {
                max_last_upload_timestamp_millis = published.timestamp_millis();
            }
        }

        self.update_channel_video_stats(&channel_id, max_last_upload_timestamp_millis)
            .await?;

        Ok(())
    }

    async fn update_channel_video_stats(
        &self,
        channel_id: &str,
        max_last_upload_timestamp_millis: i64,
    ) -> Result<(), Error> {
        let videos_per_channel = self.video_repo.count(&channel_id).await?;

        self.channel_repo
            .set_video_count_last_upload(
                &channel_id,
                videos_per_channel as i64,
                max_last_upload_timestamp_millis,
            )
            .await;

        Ok(())
    }

    fn build_video_document(
        &self,
        channel_id: &str,
        entry: &Entry,
        published: DateTime<FixedOffset>,
    ) -> Document {
        let tags: Vec<String> = vec![];
        let updated = DateTime::parse_from_rfc3339(&entry.updated).unwrap();

        let vid = doc! {
            "id": entry.video_id.clone(),
            "title": entry.title.clone(),
            "description": entry.group.description.clone(),
            "publishedAt": mongodb::bson::DateTime::from_millis(
                published.timestamp_millis(),
            ),
            "updatedAt": mongodb::bson::DateTime::from_millis(
                updated.timestamp_millis(),
            ),
            "views": entry.group.community.statistics.views,
            "channel": channel_id.clone(),
            "geoChecked": false,
            "tags": tags,
        };

        vid
    }
}

async fn load_and_parse_video_feed(channel_id: &str) -> Result<YoutubeVideoFeedResponse, Error> {
    let feed_url = format!("{}?channel_id={}", YOUTUBE_VIDEO_FEED_BASE_URL, channel_id);

    let xml = reqwest::get(&feed_url)
        .await?
        .text()
        .await?
        .replace("yt:", "yt")
        .replace("media:", "media");

    let channel_feed = from_str::<YoutubeVideoFeedResponse>(&xml).expect(&feed_url);

    Ok(channel_feed)
}
