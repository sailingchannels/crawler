use anyhow::Error;
use chrono::Utc;
use mongodb::{
    bson::{doc, Document},
    options::UpdateOptions,
    Client, Collection,
};

use crate::utils::{consts::ONE_DAYS_IN_SECONDS, db::get_db_name};

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

    pub async fn get_last_discovery_crawl(&self) -> Result<i64, Error> {
        let doc = self
            .collection
            .find_one(doc! {"_id": "lastDiscoveryCrawl"}, None)
            .await?;

        let default_value = Utc::now().timestamp() - ((ONE_DAYS_IN_SECONDS + 1) as i64);

        match doc {
            None => Ok(default_value),
            Some(d) => {
                let value = d.get_i64("value").unwrap_or(default_value);
                Ok(value)
            }
        }
    }

    pub async fn set_last_discovery_crawl(&self, last_crawl: i64) {
        let update = doc! {
            "$set": {
                "value": last_crawl,
            }
        };

        let update_options = UpdateOptions::builder().upsert(true).build();

        self.collection
            .update_one(doc! {"_id": "lastDiscoveryCrawl"}, update, update_options)
            .await
            .unwrap();
    }
}
