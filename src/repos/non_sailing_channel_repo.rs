
use mongodb::bson::{doc, Document};
use mongodb::{Client, Collection};

pub struct NonSailingChannelRepository {
    collection: Collection<Document>,
}

impl NonSailingChannelRepository {
    pub fn new(client: &Client) -> NonSailingChannelRepository {
        let db = client.database("sailing-channels");
        let channels = db.collection::<Document>("nonsailingchannels");

        NonSailingChannelRepository {
            collection: channels,
        }
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
