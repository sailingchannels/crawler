use anyhow::Error;
use log::info;
use rand::Rng;

use crate::models::youtube_statistics_response::{
    YoutubeStatisticsItem, YoutubeStatisticsResponse,
};

const BASE_URL: &str = "https://www.googleapis.com/youtube/v3/";

pub struct YoutubeService {
    api_keys: Vec<String>,
}

impl YoutubeService {
    pub fn new(api_keys: Vec<String>) -> YoutubeService {
        YoutubeService { api_keys }
    }

    pub async fn get_channel_details(
        &self,
        channel_id: &str,
    ) -> Result<YoutubeStatisticsItem, Error> {
        let url = format!(
            "{}channels?part=snippet,brandingSettings,statistics&id={}&key={}",
            BASE_URL,
            channel_id,
            self.get_api_key()
        );

        info!("{}", url);
        let resp = reqwest::get(url)
            .await?
            .json::<YoutubeStatisticsResponse>()
            .await?;

        Ok(resp.items[0].clone())
    }

    fn get_api_key(&self) -> String {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.api_keys.len());

        self.api_keys[index].clone()
    }
}
