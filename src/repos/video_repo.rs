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
        let channels = db.collection::<Document>("videos");

        VideoRepository {
            collection: channels,
        }
    }

    pub async fn delete_by_channel(&self, channel_id: String) -> Result<(), anyhow::Error> {
        self.collection
            .delete_many(doc! {"channel": channel_id}, None)
            .await?;

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
