use anyhow::Error;
use log::{debug, error};
use rand::Rng;
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;

use crate::models::detect_language_response::DetectLanguageResponse;

const BASE_URL: &str = "https://ws.detectlanguage.com/0.2/detect";
const API_RETRIES: usize = 5;
const EXPONENTIAL_BACKOFF_BASE: u64 = 100;

pub struct DetectLanguageService {
    api_keys: Vec<String>,
}

impl DetectLanguageService {
    pub fn new(api_keys: Vec<String>) -> DetectLanguageService {
        DetectLanguageService { api_keys }
    }

    pub async fn detect_language(&self, text: &str) -> Option<String> {
        let retry_strategy = ExponentialBackoff::from_millis(EXPONENTIAL_BACKOFF_BASE)
            .map(jitter)
            .take(API_RETRIES);

        let result = Retry::spawn(retry_strategy, || self.call_detect_language_api(text)).await;

        match result {
            Ok(response) => {
                if response.data.detections.len() > 0
                    && response.data.detections[0].is_reliable == true
                {
                    Some(response.data.detections[0].language.to_lowercase())
                } else {
                    None
                }
            }
            Err(err) => {
                error!("Error while calling detect language api: {}", err);
                None
            }
        }
    }

    async fn call_detect_language_api(&self, text: &str) -> Result<DetectLanguageResponse, Error> {
        let url = format!("{}?q={}", BASE_URL, text);
        let api_key = self.get_api_key();

        let client = reqwest::Client::new();
        let resp = client
            .post(url)
            .header("Authorization", format!("Bearer: {}", api_key))
            .send()
            .await?
            .json::<DetectLanguageResponse>()
            .await?;

        Ok(resp)
    }

    fn get_api_key(&self) -> String {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.api_keys.len());

        self.api_keys[index].clone()
    }
}
