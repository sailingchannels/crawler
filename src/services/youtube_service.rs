use anyhow::Error;
use log::info;

use crate::{
    models::{
        youtube_channel_details::{YouTubeChannelDetails, YoutubeStatisticsItem},
        youtube_channel_subscriptions::{
            YouTubeChannelSubscriptionSnippet, YoutubeChannelSubscriptions,
        },
    },
    repos::apikeys_repo::ApiKeyRepository,
};

const BASE_URL: &str = "https://www.googleapis.com/youtube/v3/";

pub struct YoutubeService {
    apikey_repo: ApiKeyRepository,
}

impl YoutubeService {
    pub fn new(apikey_repo: ApiKeyRepository) -> YoutubeService {
        YoutubeService { apikey_repo }
    }

    pub async fn get_channel_details(
        &self,
        channel_id: &str,
    ) -> Result<YoutubeStatisticsItem, Error> {
        let api_key = self.apikey_repo.get_least_used_api_key().await?;

        let url = format!(
            "{}channels?part=snippet,brandingSettings,statistics&id={}&key={}",
            BASE_URL, channel_id, api_key.key
        );

        let resp = reqwest::get(url)
            .await?
            .json::<YouTubeChannelDetails>()
            .await?;

        self.apikey_repo.update_usage(&api_key).await?;

        match resp.items {
            Some(items) => Ok(items[0].clone()),
            None => Err(Error::msg("No items found")),
        }
    }

    pub async fn get_channel_subscriptions(
        &self,
        channel_id: &str,
    ) -> Result<Vec<YouTubeChannelSubscriptionSnippet>, Error> {
        let mut page_token: Option<String> = None;
        let mut snippets = vec![];

        loop {
            let response = self
                .get_channel_subscriptions_page(channel_id, page_token)
                .await?;

            let response_snippets = response
                .items
                .into_iter()
                .filter(|item| item.snippet.resource_id.kind.eq(&"youtube#channel"))
                .map(|item| item.snippet)
                .collect::<Vec<YouTubeChannelSubscriptionSnippet>>();

            snippets.extend(response_snippets);
            page_token = response.next_page_token;

            if page_token.is_none() {
                break;
            }
        }

        Ok(snippets)
    }

    async fn get_channel_subscriptions_page(
        &self,
        channel_id: &str,
        page_token: Option<String>,
    ) -> Result<YoutubeChannelSubscriptions, Error> {
        let api_key = self.apikey_repo.get_least_used_api_key().await?;

        let mut url = format!(
            "{}subscriptions?part=snippet&maxResults=50&channelId={}&key={}",
            BASE_URL, channel_id, api_key.key
        );

        if page_token.is_some() {
            url = format!("{}&pageToken={}", url, page_token.unwrap());
        }

        let resp = reqwest::get(url)
            .await?
            .json::<YoutubeChannelSubscriptions>()
            .await?;

        self.apikey_repo.update_usage(&api_key).await?;

        Ok(resp)
    }
}
