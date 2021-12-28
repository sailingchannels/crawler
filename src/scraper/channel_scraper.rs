use anyhow::Error;
use chrono::{DateTime, Duration, Utc};
use log::{debug, info};
use mongodb::bson::doc;

use crate::{
    repos::{
        channel_repo::ChannelRepository, non_sailing_channel_repo::NonSailingChannelRepository,
    },
    services::youtube_service::YoutubeService,
    utils::keyword_utils,
};

const ONE_DAY_IN_SECONDS: i64 = 86400;

pub struct ChannelScraper {
    channel_repo: ChannelRepository,
    non_sailing_channel_repo: NonSailingChannelRepository,
    youtube_service: YoutubeService,
    sailing_terms: Vec<String>,
    blacklisted_channel_ids: Vec<String>,
}

impl ChannelScraper {
    pub fn new(
        channel_repo: ChannelRepository,
        non_sailing_channel_repo: NonSailingChannelRepository,
        sailing_terms: Vec<String>,
        blacklisted_channel_ids: Vec<String>,
        youtube_api_keys: Vec<String>,
    ) -> ChannelScraper {
        ChannelScraper {
            channel_repo,
            non_sailing_channel_repo,
            youtube_service: YoutubeService::new(youtube_api_keys),
            sailing_terms,
            blacklisted_channel_ids,
        }
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

        let channel_details = self
            .youtube_service
            .get_channel_details(&channel_id)
            .await?;

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
            "id": channel_id.to_string(),
            "title": channel_details.snippet.title.to_string(),
            "description": description,
            "publishedAt": mongodb::bson::DateTime::from_millis(
                published_date.timestamp_millis(),
            ),
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

        println!("{:?}", channel);

        Ok(())
    }

    async fn should_crawl_now(&self, channel_id: &str) -> bool {
        let last_crawl_default = Utc::now() - Duration::seconds(ONE_DAY_IN_SECONDS + 1);

        let last_crawl = self
            .channel_repo
            .get_last_crawl_date(channel_id)
            .await
            .unwrap_or(mongodb::bson::DateTime::from_millis(
                last_crawl_default.timestamp_millis(),
            ));

        let current_timestamp = Utc::now().timestamp();
        let should_crawl =
            current_timestamp - (last_crawl.timestamp_millis() / 1000) < ONE_DAY_IN_SECONDS;

        should_crawl
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
            self.channel_repo.delete(&channel_id).await.unwrap();
        }

        has_sailing_term
    }
}
