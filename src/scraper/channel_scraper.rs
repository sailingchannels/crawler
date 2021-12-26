use anyhow::Error;
use chrono::{Duration, Utc};
use log::info;
use mongodb::bson::doc;

use crate::{
    repos::{
        channel_repo::ChannelRepository, non_sailing_channel_repo::NonSailingChannelRepository,
    },
    services::youtube_service::YoutubeService,
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
    ) -> ChannelScraper {
        ChannelScraper {
            channel_repo,
            non_sailing_channel_repo,
            youtube_service: YoutubeService::new(),
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

        let statistics_result = self
            .youtube_service
            .get_statistics(&channel_id)
            .await
            .expect("Error getting statistics");

        let has_sailing_term = self
            .has_sailing_term(
                &channel_id,
                &statistics_result.snippet.title,
                &statistics_result.snippet.description,
                ignore_sailing_terms,
            )
            .await;

        if has_sailing_term == false
            || statistics_result.statistics.video_count.parse::<i32>()? == 0
        {
            return Ok(());
        }

        let subscriber_count = match statistics_result.statistics.subscriber_count {
            Some(subscriber_count) => subscriber_count.parse::<i64>()?,
            None => 0,
        };

        let mut channel = doc! {
            "id": channel_id.to_string(),
            "title": statistics_result.snippet.title.to_string(),
            "description": statistics_result.snippet.description.to_string(),
            "publishedAt": calendar.timegm(pd.utctimetuple()),
            "thumbnail": statistics_result.snippet.thumbnails.default.url.to_string(),
            "subscribers": subscriber_count,
            "views": int(stats["viewCount"]),
            "subscribersHidden": bool(stats["hiddenSubscriberCount"]),
            "lastCrawl": datetime.now()
        };

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
            self.channel_repo.delete(&channel_id).await;
        }

        has_sailing_term
    }
}
