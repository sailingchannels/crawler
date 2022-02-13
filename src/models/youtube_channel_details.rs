use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeChannelDetails {
    pub kind: String,
    pub etag: String,
    pub page_info: PageInfo,
    pub items: Option<Vec<YoutubeStatisticsItem>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub total_results: i64,
    pub results_per_page: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeStatisticsItem {
    pub kind: Option<String>,
    pub etag: String,
    pub id: String,
    pub snippet: Snippet,
    pub statistics: Statistics,
    pub branding_settings: BrandingSettings,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    pub title: String,
    pub description: Option<String>,
    pub custom_url: Option<String>,
    pub published_at: String,
    pub thumbnails: Thumbnails,
    pub localized: Localized,
    pub country: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnails {
    pub default: Default,
    pub medium: Medium,
    pub high: High,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Default {
    pub url: String,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Medium {
    pub url: String,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct High {
    pub url: String,
    pub width: i64,
    pub height: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Localized {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    pub view_count: String,
    pub subscriber_count: Option<String>,
    pub hidden_subscriber_count: bool,
    pub video_count: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrandingSettings {
    pub channel: Channel,
    pub image: Option<Image>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    pub title: String,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub tracking_analytics_account_id: Option<String>,
    pub moderate_comments: Option<bool>,
    pub unsubscribed_trailer: Option<String>,
    pub country: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub banner_external_url: String,
}
