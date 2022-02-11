use anyhow::Error;
use chrono::Utc;
use futures::stream::TryStreamExt;
use mongodb::bson::{doc, Document};
use mongodb::options::{FindOneOptions, FindOptions};
use mongodb::{Client, Collection};

use crate::utils::db::get_db_name;

pub struct ChannelRepository {
    collection: Collection<Document>,
}

impl ChannelRepository {
    pub fn new(client: &Client, environment: &str) -> ChannelRepository {
        let db = client.database(&get_db_name(&environment));
        let channels = db.collection::<Document>("channels");

        ChannelRepository {
            collection: channels,
        }
    }

    pub async fn get_all_ids(&self) -> Result<Vec<String>, Error> {
        let find_options = FindOptions::builder().projection(doc! { "_id": 1 }).build();
        let cursor = self.collection.find(None, find_options).await?;
        let channels: Vec<Document> = cursor.try_collect().await?;

        let channel_ids = channels
            .iter()
            .map(|doc| doc.get_str("_id").unwrap().to_string())
            .collect();

        Ok(channel_ids)
    }

    pub async fn get_ids_last_crawled_before(
        &self,
        last_crawl_before: chrono::DateTime<Utc>,
        _last_upload_after: chrono::DateTime<Utc>,
    ) -> Result<Vec<String>, Error> {
        let find_options = FindOptions::builder().projection(doc! { "_id": 1 }).build();
        let query = doc! {
            "lastCrawl": {
                "$lt": mongodb::bson::DateTime::from_millis(
                    last_crawl_before.timestamp_millis(),
                )
            }
        };

        let cursor = self.collection.find(query, find_options).await?;
        let channels: Vec<Document> = cursor.try_collect().await?;

        let channel_ids = channels
            .iter()
            .map(|doc| doc.get_str("_id").unwrap().to_string())
            .collect();

        Ok(channel_ids)
    }

    pub async fn get_detected_language(&self, id: &str) -> Result<String, Error> {
        let find_one_options = FindOneOptions::builder()
            .projection(doc! {"detectedLanguage": 1})
            .build();

        let channel = self
            .collection
            .find_one(doc! {"_id": id}, find_one_options)
            .await?
            .unwrap();

        let detected_language = channel.get_str("detectedLanguage")?;

        Ok(detected_language.to_string())
    }

    pub async fn delete(&self, id: &str) -> Result<(), anyhow::Error> {
        self.collection.delete_one(doc! {"_id": id}, None).await?;

        Ok(())
    }

    pub async fn upsert(&self, id: &str, channel: Document) {
        let update_options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();

        self.collection
            .update_one(doc! {"_id": id}, doc! {"$set": channel}, update_options)
            .await
            .unwrap();
    }

    pub async fn set_video_count_last_upload(
        &self,
        id: &str,
        video_count: i64,
        last_upload_timestamp: i64,
    ) {
        self.collection
            .update_one(
                doc! {"_id": id},
                doc! {
                    "$set": {
                        "videoCount": video_count,
                        "lastUploadAt": last_upload_timestamp,
                        "lastVideoCrawl": mongodb::bson::DateTime::now()
                    }
                },
                None,
            )
            .await
            .unwrap();
    }

    pub async fn set_scrape_error(&self, id: &str, error: String) {
        self.collection
            .update_one(
                doc! {"_id": id},
                doc! {
                    "$set": {
                        "scrapeError": {
                            "at": mongodb::bson::DateTime::now(),
                            "error": error
                        }
                    }
                },
                None,
            )
            .await
            .unwrap();
    }
}
