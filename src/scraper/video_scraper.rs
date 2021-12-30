use anyhow::Error;
use mongodb::bson::doc;
use quick_xml::de::from_str;

use crate::models::youtube_video_feed_response::YoutubeVideoFeedResponse;

const YOUTUBE_VIDEO_FEED_BASE_URL: &str = "https://www.youtube.com/feeds/videos.xml";

pub struct VideoScraper {}

impl VideoScraper {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn scrape(&self, channel_id: String) -> Result<(), Error> {
        let channel_url = format!("{}?channel_id={}", YOUTUBE_VIDEO_FEED_BASE_URL, channel_id);

        let xml = reqwest::get(&channel_url).await?.text().await?;
        let channel_feed: YoutubeVideoFeedResponse = from_str(&xml)?;

        let mut videos = vec![];

        for entry in channel_feed.feed.entry.iter() {
            let vid = doc! {
                "id": entry.video_id.text.clone(),
                /*"title": item.title()?,
                "description": feedItem["media:group"]["media:description"],
                "publishedAt": publishedAt,
                "updatedAt": updatedAt,
                "views": int(feedItem["media:group"]["media:community"]["media:statistics"]["@views"]),
                "channel": feedItem["yt:channelId"],
                "geoChecked": false,
                "tags": vec![]*/
            };

            println!("{:?}", vid);

            videos.push(vid);
        }
        /*

        for item in channel_feed.items().iter() {
            let video_id = item.guid().unwrap().value();
            let video_url = format!("https://www.youtube.com/watch?v={}", video_id);


        }*/

        Ok(())
    }
}
