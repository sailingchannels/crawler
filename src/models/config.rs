use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct CrawlerConfig {
    pub additional: bool,
    pub discovery: bool,
    pub video: bool,
    pub channel: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub mongo_connection_string: String,
    pub environment: String,
    pub log_level: String,
    pub crawler: CrawlerConfig,
}
