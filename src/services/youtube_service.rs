use anyhow::Error;

use crate::{
    models::youtube_channel_details::{YouTubeChannelDetails, YoutubeStatisticsItem},
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
}
