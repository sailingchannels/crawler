use mongodb::bson::{doc, Document};
use mongodb::{Client, Collection};

use crate::utils::db::get_db_name;

pub struct ViewRepository {
    collection: Collection<Document>,
}

impl ViewRepository {
    pub fn new(client: &Client, environment: &str) -> ViewRepository {
        let db = client.database(&get_db_name(&environment));
        let channels = db.collection::<Document>("views");

        ViewRepository {
            collection: channels,
        }
    }

    pub async fn delete_by_channel(&self, channel_id: &str) -> Result<(), anyhow::Error> {
        self.collection
            .delete_many(doc! {"_id": {"channel": channel_id}}, None)
            .await?;

        Ok(())
    }

    pub async fn upsert(&self, id: Document, view: Document) -> Result<(), anyhow::Error> {
        let update_options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();

        self.collection
            .update_one(doc! {"_id": id}, doc! {"$set": view}, update_options)
            .await?;

        Ok(())
    }
}
