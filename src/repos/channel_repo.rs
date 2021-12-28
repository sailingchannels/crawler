use anyhow::Error;
use futures::stream::TryStreamExt;
use mongodb::bson::{doc, DateTime, Document};
use mongodb::options::{FindOneOptions, FindOptions};
use mongodb::{Client, Collection};

pub struct ChannelRepository {
    collection: Collection<Document>,
}

impl ChannelRepository {
    pub fn new(client: &Client) -> ChannelRepository {
        let db = client.database("sailing-channels");
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

    pub async fn get_last_crawl_date(&self, id: &str) -> Result<DateTime, Error> {
        let find_one_options = FindOneOptions::builder()
            .projection(doc! {"lastCrawl": 1})
            .build();

        let channel = self
            .collection
            .find_one(doc! {"_id": id}, find_one_options)
            .await?
            .unwrap();

        let last_crawl = channel.get_datetime("lastCrawl")?;

        Ok(last_crawl.clone())
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
}
