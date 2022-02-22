use anyhow::Error;
use mongodb::{
    bson::{doc, Document},
    options::UpdateOptions,
    Client, Collection,
};

use crate::utils::db::get_db_name;

pub struct SettingsRepository {
    collection: Collection<Document>,
}

impl SettingsRepository {
    pub fn new(client: &Client, environment: &str) -> SettingsRepository {
        let db = client.database(&get_db_name(&environment));
        let settings = db.collection::<Document>("settings");

        SettingsRepository {
            collection: settings,
        }
    }

    pub async fn get_last_subscriber_crawl(&self) -> Result<i64, Error> {
        let doc = self
            .collection
            .find_one(doc! {"_id": "lastSubscriberCrawl"}, None)
            .await?;

        let value = doc.unwrap().get_i64("value").unwrap();
        Ok(value)
    }

    pub async fn set_last_subscriber_crawl(&self, last_crawl: i64) {
        let update = doc! {
            "$set": {
                "value": last_crawl,
            }
        };

        let update_options = UpdateOptions::builder().upsert(true).build();

        self.collection
            .update_one(doc! {"_id": "lastSubscriberCrawl"}, update, update_options)
            .await
            .unwrap();
    }
}
