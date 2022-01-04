use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub mongo_connection_string: String,
    pub youtube_api_keys: Vec<String>,
    pub youtube_video_api_keys: Vec<String>,
    pub environment: String,
    pub log_level: String,
}
