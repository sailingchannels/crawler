use mongodb::bson::{doc, Document};
use mongodb::{Client, Collection};

pub struct SubscriberRepository {
    collection: Collection<Document>,
}

impl SubscriberRepository {
    pub fn new(client: &Client) -> SubscriberRepository {
        let db = client.database("sailing-channels");
        let channels = db.collection::<Document>("subscribers");

        SubscriberRepository {
            collection: channels,
        }
    }

    pub async fn delete(&self, id: String) -> Result<(), anyhow::Error> {
        self.collection.delete_one(doc! {"_id": id}, None).await?;

        Ok(())
    }

    pub async fn upsert(&self, id: String, view: Document) -> Result<(), anyhow::Error> {
        let update_options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();

        self.collection
            .update_one(doc! {"_id": id}, doc! {"$set": view}, update_options)
            .await?;

        Ok(())
    }
}
