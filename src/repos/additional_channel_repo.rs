use anyhow::Error;
use futures::stream::TryStreamExt;
use mongodb::bson::{doc, Document};
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

    pub async fn get_all(&self) -> Result<Vec<Document>, Error> {
        let cursor = self.collection.find(None, None).await?;
        let additional_channels: Vec<Document> = cursor.try_collect().await?;

        Ok(additional_channels)
    }

    pub async fn delete_one(&self, id: String) -> Result<(), Error> {
        let filter = doc! {"_id": id};
        self.collection.delete_one(filter, None).await?;

        Ok(())
    }
}
