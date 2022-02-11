use std::collections::HashMap;

use anyhow::Error;
use chrono::{TimeZone, Utc};
use futures::stream::TryStreamExt;
use mongodb::bson::{doc, Document};
use mongodb::options::FindOptions;
use mongodb::{Client, Collection};

use crate::utils::db::get_db_name;

pub struct VideoRepository {
    collection: Collection<Document>,
}

impl VideoRepository {
    pub fn new(client: &Client, environment: &str) -> VideoRepository {
        let db = client.database(&get_db_name(&environment));
        let channels = db.collection::<Document>("videos");

        VideoRepository {
            collection: channels,
        }
    }

    pub async fn get_updated_lookup(
        &self,
        channel_id: &str,
    ) -> Result<HashMap<String, chrono::DateTime<Utc>>, Error> {
        let find_options = FindOptions::builder()
            .projection(doc! {
                "_id" : 1,
                "updatedAt" : 1
            })
            .build();

        let query = doc! {"channel": channel_id};

        let cursor = self.collection.find(query, find_options).await?;
        let videos: Vec<Document> = cursor.try_collect().await?;

        let video_updated_lookup = videos
            .iter()
            .filter(|doc| doc.contains_key("updatedAt") && doc.get_i64("updatedAt").is_ok())
            .map(|doc| {
                let id = doc.get_str("_id").unwrap().to_string();
                let updated_at = doc.get_i64("updatedAt").unwrap();

                (id, Utc.timestamp(updated_at as i64, 0))
            })
            .collect::<HashMap<String, chrono::DateTime<Utc>>>();

        Ok(video_updated_lookup)
    }

    pub async fn delete_all_by_channel(&self, channel_id: &str) -> Result<(), anyhow::Error> {
        self.collection
            .delete_many(doc! {"channel": channel_id}, None)
            .await?;

        Ok(())
    }

    pub async fn upsert(&self, id: &str, video_doc: Document) -> Result<(), anyhow::Error> {
        let update_options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();

        self.collection
            .update_one(doc! {"_id": id}, doc! {"$set": video_doc}, update_options)
            .await?;

        Ok(())
    }

    pub async fn count(&self, channel_id: &str) -> Result<u64, anyhow::Error> {
        let count = self
            .collection
            .count_documents(doc! {"channel": channel_id}, None)
            .await?;

        Ok(count)
    }
}
