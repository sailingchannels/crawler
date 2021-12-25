use anyhow::Error;
use futures::stream::TryStreamExt;
use mongodb::bson::{doc, DateTime, Document};
use mongodb::options::{FindOneOptions, FindOptions};
use mongodb::{Client, Collection};

pub struct VideoRepository {
    collection: Collection<Document>,
}

impl VideoRepository {
    pub fn new(client: &Client) -> VideoRepository {
        let db = client.database("sailing-channels");
        let channels = db.collection::<Document>("channels");

        VideoRepository {
            collection: channels,
        }
    }

    pub async fn get_last_crawl_date(&self, id: String) -> Result<DateTime, Error> {
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

    pub async fn delete(&self, channel_id: String) -> Result<(), anyhow::Error> {
        self.collection.delete_one(doc! {"_id": id}, None).await?;

        Ok(())
    }

    pub async fn upsert(&self, id: String, channel: Document) -> Result<(), anyhow::Error> {
        let update_options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();

        self.collection
            .update_one(doc! {"_id": id}, doc! {"$set": channel}, update_options)
            .await?;

        Ok(())
    }
}
