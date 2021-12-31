use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeVideoFeedResponse {
    #[serde(rename = "entry", default)]
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    #[serde(rename = "ytvideoId")]
    pub video_id: String,
    pub title: String,
    pub published: String,
    pub updated: String,
    #[serde(rename = "mediagroup")]
    pub group: MediaGroup,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaGroup {
    #[serde(rename = "mediatitle")]
    pub title: String,
    #[serde(rename = "mediadescription")]
    pub description: String,
    #[serde(rename = "mediacommunity")]
    pub community: MediaCommunity,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaCommunity {
    #[serde(rename = "mediastatistics")]
    pub statistics: MediaStatistics,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaStatistics {
    pub views: i64,
}
