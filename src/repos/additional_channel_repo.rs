use futures::stream::TryStreamExt;
use mongodb::bson::Document;
use mongodb::{Client, Collection};

pub struct AdditionalChannelRepository {
    collection: Collection<Document>,
}

impl AdditionalChannelRepository {
    pub fn new(client: &Client) -> AdditionalChannelRepository {
        let db = client.database("sailing-channels");
        let feeds = db.collection::<Document>("additional");

        AdditionalChannelRepository { collection: feeds }
    }

    pub async fn get_all(&self) -> Result<Vec<Document>, anyhow::Error> {
        let cursor = self.collection.find(None, None).await?;
        let additional_channels: Vec<Document> = cursor.try_collect().await?;

        Ok(additional_channels)
    }
}