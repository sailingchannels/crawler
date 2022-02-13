use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub mongo_connection_string: String,
    pub environment: String,
    pub log_level: String,
}
