use anyhow::Error;
use chrono::{DateTime, Utc};
use chrono_tz::{Tz, US::Pacific};
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use mongodb::{Client, Collection};

use crate::models::apikey::ApiKey;
use crate::utils::db::get_db_name;

pub struct ApiKeyRepository {
    collection: Collection<ApiKey>,
}

impl ApiKeyRepository {
    pub fn new(client: &Client, environment: &str) -> ApiKeyRepository {
        let db = client.database(&get_db_name(&environment));
        let channels = db.collection::<ApiKey>("apikeys");

        ApiKeyRepository {
            collection: channels,
        }
    }

    pub async fn get_least_used_api_key(&self) -> Result<ApiKey, Error> {
        let find_options = FindOneOptions::builder()
            .sort(doc! { "used_quota": 1 })
            .build();

        let doc = self
            .collection
            .find_one(doc! {}, find_options)
            .await?
            .unwrap();

        Ok(doc)
    }

    pub async fn update_usage(&self, api_key: &ApiKey) -> Result<(), Error> {
        let pacific_now: DateTime<Tz> = Utc::now().with_timezone(&Pacific);
        let pacific_date = pacific_now
            .format("%Y%m%d")
            .to_string()
            .parse::<i32>()
            .unwrap();

        let mut update = doc! {
            "$inc": {
                "used_quota": 1
            }
        };

        if pacific_date > api_key.pdt_day {
            update = doc! {
                "$set": {
                    "used_quota": 1,
                    "pdt_day": pacific_date,
                }
            };
        }

        self.collection
            .update_one(doc! {"_id": &api_key.key}, update, None)
            .await?;

        Ok(())
    }
}
