use anyhow::{anyhow, Error};
use rand::Rng;

use crate::models::{
    youtube_channel_details::{YouTubeChannelDetails, YoutubeStatisticsItem},
    youtube_video_details::{YouTubeVideoDetails, YouTubeVideoItem},
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

        let resp = reqwest::get(url)
            .await?
            .json::<YouTubeChannelDetails>()
            .await?;

        Ok(resp.items[0].clone())
    }

    pub async fn get_video_details(&self, video_id: &str) -> Result<YouTubeVideoItem, Error> {
        let url = format!(
            "{}videos?part=snippet,statistics,status&id={}&key={}",
            BASE_URL,
            video_id,
            self.get_api_key()
        );

        let resp = reqwest::get(url).await?;

        if !resp.status().is_success() {
            return Err(anyhow!(
                "Failed to get video details. Possibly out of quota"
            ));
        }

        let video_details = resp.json::<YouTubeVideoDetails>().await?;

        Ok(video_details.items[0].clone())
    }

    fn get_api_key(&self) -> String {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.api_keys.len());

        self.api_keys[index].clone()
    }
}
