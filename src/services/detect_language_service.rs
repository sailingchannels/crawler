use anyhow::Error;
use log::info;
use rand::Rng;

use crate::models::detect_language_response::DetectLanguageResponse;

const BASE_URL: &str = "https://ws.detectlanguage.com/0.2/detect";

pub struct DetectLanguageService {
    api_keys: Vec<String>,
}

impl DetectLanguageService {
    pub fn new(api_keys: Vec<String>) -> DetectLanguageService {
        DetectLanguageService { api_keys }
    }

    pub async fn detect_language(&self, text: &str) -> Result<Option<String>, Error> {
        let url = format!("{}q={}", BASE_URL, text);

        let client = reqwest::Client::new();
        let resp = client
            .post(url)
            .header("Authorization", format!("Bearer: {}", self.get_api_key()))
            .send()
            .await?
            .json::<DetectLanguageResponse>()
            .await?;

        Ok(Some(resp.data.detections[0].language.to_lowercase()))
    }

    fn get_api_key(&self) -> String {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.api_keys.len());

        self.api_keys[index].clone()
    }
}
