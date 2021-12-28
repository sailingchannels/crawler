use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub youtube_api_keys: Vec<String>,
    pub mongo_connection_string: String,
}
