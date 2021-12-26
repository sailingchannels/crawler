use anyhow::Error;

use crate::models::youtube_statistics_response::{
    YoutubeStatisticsItem, YoutubeStatisticsResponse,
};

const BASE_URL: &str = "https://www.googleapis.com/youtube/v3/";

pub struct YoutubeService {
    api_keys: Vec<String>,
}

impl YoutubeService {
    pub fn new() -> YoutubeService {
        YoutubeService { api_keys: vec![] }
    }

    pub async fn get_statistics(&self, channel_id: &str) -> Result<YoutubeStatisticsItem, Error> {
        let url = format!(
            "{}channels?part=statistics&id={}&key={}",
            BASE_URL,
            channel_id,
            self.get_api_key()
        );

        let resp = reqwest::get(url)
            .await?
            .json::<YoutubeStatisticsResponse>()
            .await?;

        Ok(resp.items[0].clone())
    }

    fn get_api_key(&self) -> String {
        "".to_string()
    }
}
