use std::collections::HashMap;

use anyhow::{anyhow, Error};
use chrono::{DateTime, FixedOffset, Utc};
use log::info;
use mongodb::bson::{doc, Document};
use quick_xml::de::from_str;

use crate::{
    models::{
        youtube_video_details::YouTubeVideoItem,
        youtube_video_feed_response::{Entry, YoutubeVideoFeedResponse},
    },
    repos::{channel_repo::ChannelRepository, video_repo::VideoRepository},
    services::youtube_service::YoutubeService,
};

const YOUTUBE_VIDEO_FEED_BASE_URL: &str = "https://www.youtube.com/feeds/videos.xml";

pub struct VideoScraper {
    video_repo: VideoRepository,
    channel_repo: ChannelRepository,
    youtube_service: YoutubeService,
}

impl VideoScraper {
    pub fn new(
        video_repo: VideoRepository,
        channel_repo: ChannelRepository,
        youtube_api_keys: Vec<String>,
        youtube_video_api_keys: Vec<String>,
    ) -> Self {
        Self {
            video_repo,
            channel_repo,
            youtube_service: YoutubeService::new(youtube_api_keys, youtube_video_api_keys),
        }
    }

    pub async fn scrape(&self, channel_id: String) -> Result<(), Error> {
        let channel_feed = load_and_parse_video_feed(&channel_id).await?;
        let updated_lookup = self.video_repo.get_updated_lookup(&channel_id).await?;

        let mut max_last_upload_timestamp: i64 = 0;

        for entry in channel_feed.entries.iter() {
            let published = DateTime::parse_from_rfc3339(&entry.published)?;
            let updated = DateTime::parse_from_rfc3339(&entry.updated).unwrap();

            let should_update = should_update_video(&updated_lookup, entry, updated);
            if !should_update {
                continue;
            }

            let video_details = self
                .youtube_service
                .get_video_details(&entry.video_id)
                .await?;

            if video_details.status.privacy_status.ne("public") {
                self.video_repo.delete(&channel_id).await?;

                info!(
                    "Video {} is private, delete if exists and skipping",
                    entry.video_id
                );

                continue;
            }

            let vid =
                self.build_video_document(&channel_id, &entry, published, updated, &video_details);

            if published.timestamp() > max_last_upload_timestamp {
                max_last_upload_timestamp = published.timestamp();
            }

            info!("Updating video {}", entry.video_id);

            self.video_repo.upsert(&entry.video_id, vid).await?;
        }

        self.update_channel_video_stats(&channel_id, max_last_upload_timestamp)
            .await?;

        Ok(())
    }

    async fn update_channel_video_stats(
        &self,
        channel_id: &str,
        max_last_upload_timestamp: i64,
    ) -> Result<(), Error> {
        let videos_per_channel = self.video_repo.count(&channel_id).await?;

        self.channel_repo
            .set_video_count_last_upload(
                &channel_id,
                videos_per_channel as i64,
                max_last_upload_timestamp,
            )
            .await;

        Ok(())
    }

    fn build_video_document(
        &self,
        channel_id: &str,
        entry: &Entry,
        published: DateTime<FixedOffset>,
        updated: DateTime<FixedOffset>,
        video_details: &YouTubeVideoItem,
    ) -> Document {
        let views = video_details
            .statistics
            .view_count
            .parse::<i64>()
            .unwrap_or_default();

        let likes = match &video_details.statistics.like_count {
            Some(likes) => likes.parse::<i64>().unwrap_or_default(),
            None => 0,
        };

        let comments = video_details
            .statistics
            .comment_count
            .parse::<i64>()
            .unwrap_or_default();

        let mut vid = doc! {
            "_id": entry.video_id.clone(),
            "title": entry.title.clone(),
            "description": entry.group.description.clone(),
            "publishedAt": published.timestamp(),
            "updatedAt": updated.timestamp(),
            "views": views,
            "likes": likes,
            "comments": comments,
            "channel": channel_id.clone(),
            "geoChecked": false,
            "tags": video_details.snippet.tags.clone().unwrap_or_default(),
        };

        if video_details.snippet.default_language.is_some() {
            vid.insert(
                "defaultLanguage",
                video_details.snippet.default_language.clone().unwrap(),
            );
        }

        vid
    }
}

fn should_update_video(
    updated_lookup: &HashMap<String, DateTime<Utc>>,
    entry: &Entry,
    updated: DateTime<FixedOffset>,
) -> bool {
    let should_update = if !updated_lookup.contains_key(&entry.video_id) {
        true
    } else {
        updated > updated_lookup[&entry.video_id]
    };

    should_update
}

async fn load_and_parse_video_feed(channel_id: &str) -> Result<YoutubeVideoFeedResponse, Error> {
    let feed_url = format!("{}?channel_id={}", YOUTUBE_VIDEO_FEED_BASE_URL, channel_id);

    let response = reqwest::get(&feed_url).await?;

    if response.status() != 200 {
        return Err(anyhow!(
            "Youtube Video Feed Response Error: {}",
            response.status()
        ));
    }

    let xml = response
        .text()
        .await?
        .replace("yt:", "yt")
        .replace("media:", "media");

    let channel_feed = from_str::<YoutubeVideoFeedResponse>(&xml).expect(&format!(
        "{}, xml string length {}",
        &feed_url,
        xml.len()
    ));

    Ok(channel_feed)
}
