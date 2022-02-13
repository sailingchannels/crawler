use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApiKey {
    #[serde(rename(deserialize = "_id"))]
    pub key: String,
    pub used_quota: i32,
    pub daily_quota: i32,
    pub pdt_day: i32,
}
