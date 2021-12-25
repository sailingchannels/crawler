use anyhow::Error;
use futures::stream::TryStreamExt;
use mongodb::bson::{doc, Document};
use mongodb::{Client, Collection};

pub struct BlacklistRepository {
    collection: Collection<Document>,
}

impl BlacklistRepository {
    pub fn new(client: &Client) -> BlacklistRepository {
        let db = client.database("sailing-channels");
        let feeds = db.collection::<Document>("additional");

        BlacklistRepository { collection: feeds }
    }

    pub async fn get_all(&self) -> Result<Vec<String>, Error> {
        let find_options = mongodb::options::FindOptions::builder()
            .projection(doc! {"_id": 1})
            .build();

        let cursor = self.collection.find(None, find_options).await?;
        let sailing_terms: Vec<Document> = cursor.try_collect().await?;

        let ids: Vec<String> = sailing_terms
            .iter()
            .map(|doc| doc.get_str("_id").unwrap().to_string())
            .collect();

        Ok(ids)
    }
}
