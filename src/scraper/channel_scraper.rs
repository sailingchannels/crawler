use anyhow::Error;
use chrono::{DateTime, Datelike, Utc};
use log::{debug, error, info, warn};
use mongodb::bson::doc;
use whatlang::detect;

use crate::{
    models::youtube_channel_details::YoutubeStatisticsItem,
    repos::{
        channel_repo::ChannelRepository, non_sailing_channel_repo::NonSailingChannelRepository,
        subscriber_repo::SubscriberRepository, video_repo::VideoRepository,
        view_repo::ViewRepository,
    },
    services::youtube_service::YoutubeService,
    utils::{consts::DEVELOPMENT, keyword_utils},
};

pub struct ChannelScraper {
    channel_repo: ChannelRepository,
    view_repo: ViewRepository,
    subscriber_repo: SubscriberRepository,
    video_repo: VideoRepository,
    non_sailing_channel_repo: NonSailingChannelRepository,
    youtube_service: YoutubeService,
    sailing_terms: Vec<String>,
    blacklisted_channel_ids: Vec<String>,
    environment: String,
}

impl ChannelScraper {
    pub fn new(
        channel_repo: ChannelRepository,
        view_repo: ViewRepository,
        subscriber_repo: SubscriberRepository,
        video_repo: VideoRepository,
        non_sailing_channel_repo: NonSailingChannelRepository,
        sailing_terms: Vec<String>,
        blacklisted_channel_ids: Vec<String>,
        youtube_api_keys: Vec<String>,
        environment: String,
    ) -> ChannelScraper {
        ChannelScraper {
            channel_repo,
            view_repo,
            subscriber_repo,
            video_repo,
            non_sailing_channel_repo,
            youtube_service: YoutubeService::new(youtube_api_keys),
            sailing_terms,
            blacklisted_channel_ids,
            environment,
        }
    }

    pub async fn scrape(
        &self,
        channel_id: String,
        ignore_sailing_terms: bool,
    ) -> Result<(), Error> {
        info!("Start scraping channel {}", channel_id);

        let channel_details = match self.load_channel_details(&channel_id).await {
            Ok(value) => value,
            Err(value) => return value,
        };

        let description = channel_details.snippet.description.unwrap_or_default();

        let has_sailing_term = self
            .has_sailing_term(
                &channel_id,
                &channel_details.snippet.title,
                &description,
                ignore_sailing_terms,
            )
            .await;

        let view_count = channel_details
            .statistics
            .view_count
            .parse::<i64>()
            .unwrap_or(0);

        if has_sailing_term == false || view_count == 0 {
            return Ok(());
        }

        let subscriber_count = match channel_details.statistics.subscriber_count {
            Some(subscriber_count) => subscriber_count.parse::<i64>()?,
            None => 0,
        };

        let published_date = DateTime::parse_from_rfc3339(&channel_details.snippet.published_at)?;

        let mut channel = doc! {
            "_id": channel_id.to_string(),
            "title": channel_details.snippet.title.to_string(),
            "description": description.to_string(),
            "publishedAt": published_date.timestamp(),
            "thumbnail": channel_details.snippet.thumbnails.default.url.to_string(),
            "subscribers": subscriber_count,
            "views": view_count,
            "subscribersHidden": channel_details.statistics.hidden_subscriber_count,
            "lastCrawl": mongodb::bson::DateTime::now(),
        };

        if channel_details.snippet.country.is_some() {
            channel.insert(
                "country",
                channel_details.snippet.country.unwrap().to_lowercase(),
            );
        }

        let keywords = keyword_utils::parse_keywords(
            &channel_details
                .branding_settings
                .channel
                .keywords
                .unwrap_or_default(),
        );

        if keywords.len() > 0 {
            channel.insert("keywords", keywords);
        }

        let language_option = self.detect_language(&channel_id, &description).await;
        match language_option {
            Some(language) => {
                channel.insert("language", language);
                channel.insert("detectedLanguage", true);
            }
            None => {}
        }

        self.store_view_count(&channel_id, view_count).await;
        self.store_subscriber_count(&channel_id, subscriber_count)
            .await;

        if self.environment.eq(DEVELOPMENT) {
            debug!("Channel {} stored", channel_id);
        } else {
            self.channel_repo.upsert(&channel_id, channel).await;
        }

        Ok(())
    }

    async fn load_channel_details(
        &self,
        channel_id: &String,
    ) -> Result<YoutubeStatisticsItem, Result<(), Error>> {
        let channel_details_result = self.youtube_service.get_channel_details(channel_id).await;
        let channel_details = match channel_details_result {
            Ok(channel_details) => channel_details,
            Err(err) => {
                error!("Failed to get channel details for {}: {}", channel_id, err);

                self.channel_repo
                    .set_scrape_error(channel_id, err.to_string())
                    .await;

                return Err(Ok(()));
            }
        };

        Ok(channel_details)
    }

    async fn has_sailing_term(
        &self,
        channel_id: &str,
        channel_title: &str,
        channel_description: &str,
        ignore_sailing_terms: bool,
    ) -> bool {
        let mut has_sailing_term = false;

        for term in &self.sailing_terms {
            if channel_title.to_lowercase().contains(term)
                || channel_description.to_lowercase().contains(term)
            {
                has_sailing_term = true;
                break;
            }
        }

        if has_sailing_term == false && ignore_sailing_terms == false {
            self.non_sailing_channel_repo.upsert(&channel_id).await;
        }

        if ignore_sailing_terms == true {
            has_sailing_term = true;
        }

        if self
            .blacklisted_channel_ids
            .contains(&channel_id.to_string())
        {
            has_sailing_term = false;
            self.delete_channel(channel_id).await.unwrap();
        }

        has_sailing_term
    }

    async fn delete_channel(&self, channel_id: &str) -> Result<(), Error> {
        self.channel_repo.delete(channel_id).await?;
        self.view_repo.delete_by_channel(channel_id).await?;
        self.subscriber_repo.delete_by_channel(channel_id).await?;
        self.video_repo.delete_all_by_channel(channel_id).await?;

        Ok(())
    }

    async fn detect_language(&self, channel_id: &str, text: &str) -> Option<String> {
        let channel_language_result = self.channel_repo.get_detected_language(channel_id).await;

        let language_detected = channel_language_result.is_ok();

        if language_detected == false && text.len() > 0 {
            match detect(text) {
                Some(language) => {
                    if language.is_reliable() {
                        return Some(language.lang().code()[..2].to_string());
                    }
                }
                None => {
                    warn!(
                        "Language detection failed for channel {} with text {}",
                        channel_id, text
                    );

                    return None;
                }
            }
        }

        None
    }

    async fn store_view_count(&self, channel_id: &str, view_count: i64) {
        let now = Utc::now();

        self.view_repo
            .upsert(
                doc! {
                    "channel": channel_id.to_string(),
                    "date": now.format("%Y%m%d").to_string().parse::<i32>().unwrap(),
                },
                doc! {
                    "year": now.year(),
                    "month": now.month(),
                    "day": now.day(),
                    "date": mongodb::bson::DateTime::from_millis(
                        now.timestamp_millis() as i64
                    ),
                    "views": view_count
                },
            )
            .await
            .expect("Failed to upsert view count");
    }

    async fn store_subscriber_count(&self, channel_id: &str, subscriber_count: i64) {
        let now = Utc::now();

        self.subscriber_repo
            .upsert(
                doc! {
                    "channel": channel_id.to_string(),
                    "date": now.format("%Y%m%d").to_string().parse::<i32>().unwrap(),
                },
                doc! {
                    "year": now.year(),
                    "month": now.month(),
                    "day": now.day(),
                    "date": mongodb::bson::DateTime::from_millis(
                        now.timestamp_millis() as i64
                    ),
                    "subscribers": subscriber_count
                },
            )
            .await
            .expect("Failed to upsert view count");
    }
}
