use mongodb::bson::{doc, DateTime, Document};
use mongodb::{Client, Collection};

use crate::utils::db::get_db_name;

pub struct NonSailingChannelRepository {
    collection: Collection<Document>,
}

impl NonSailingChannelRepository {
    pub fn new(client: &Client, environment: &str) -> NonSailingChannelRepository {
        let db = client.database(&get_db_name(&environment));
        let channels = db.collection::<Document>("nonsailingchannels");

        NonSailingChannelRepository {
            collection: channels,
        }
    }

    pub async fn upsert(&self, channel_id: &str) {
        let update_options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();

        self.collection
            .update_one(
                doc! {"_id": channel_id},
                doc! {"$set": {"_id": channel_id, "decisionMadeAt": DateTime::now()}},
                update_options,
            )
            .await
            .unwrap();
    }
}
