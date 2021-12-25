use mongodb::bson::{doc, Document};
use mongodb::{Client, Collection};

pub struct ViewRepository {
    collection: Collection<Document>,
}

impl ViewRepository {
    pub fn new(client: &Client) -> ViewRepository {
        let db = client.database("sailing-channels");
        let channels = db.collection::<Document>("views");

        ViewRepository {
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
