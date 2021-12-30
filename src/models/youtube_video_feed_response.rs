use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeVideoFeedResponse {
    pub feed: Feed,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub link: Vec<Link>,
    pub id: String,
    pub channel_id: ChannelId,
    pub title: String,
    pub author: Author,
    pub published: String,
    pub entry: Vec<Entry>,
    #[serde(rename = "_xmlns:yt")]
    pub xmlns_yt: String,
    #[serde(rename = "_xmlns:media")]
    pub xmlns_media: String,
    #[serde(rename = "_xmlns")]
    pub xmlns: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    #[serde(rename = "_rel")]
    pub rel: String,
    #[serde(rename = "_href")]
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelId {
    #[serde(rename = "__text")]
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub name: String,
    pub uri: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub video_id: VideoId,
    pub channel_id: ChannelId2,
    pub title: String,
    pub link: Vec<Link2>,
    pub author: Author2,
    pub published: String,
    pub updated: String,
    pub group: Group,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoId {
    #[serde(rename = "__text")]
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelId2 {
    #[serde(rename = "__text")]
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link2 {
    #[serde(rename = "_rel")]
    pub rel: String,
    #[serde(rename = "_href")]
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author2 {
    pub name: String,
    pub uri: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub title: Title,
    pub content: Content,
    pub thumbnail: Thumbnail,
    pub description: Description,
    pub community: Community,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title {
    #[serde(rename = "__text")]
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    #[serde(rename = "_url")]
    pub url: String,
    #[serde(rename = "_type")]
    pub type_field: String,
    #[serde(rename = "_width")]
    pub width: String,
    #[serde(rename = "_height")]
    pub height: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail {
    #[serde(rename = "_url")]
    pub url: String,
    #[serde(rename = "_width")]
    pub width: String,
    #[serde(rename = "_height")]
    pub height: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Description {
    #[serde(rename = "__text")]
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Community {
    pub star_rating: StarRating,
    pub statistics: Statistics,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarRating {
    #[serde(rename = "_count")]
    pub count: String,
    #[serde(rename = "_average")]
    pub average: String,
    #[serde(rename = "_min")]
    pub min: String,
    #[serde(rename = "_max")]
    pub max: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    #[serde(rename = "_views")]
    pub views: String,
}
