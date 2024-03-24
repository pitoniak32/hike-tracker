use anyhow::Result;

use mongodb::{bson::oid::ObjectId, Client, Collection};
use serde::{Deserialize, Serialize};

pub const MONGO_DB_NAME: &str = "hike-tracker";
pub const MONGO_COLL_NAME_TRACKERS: &str = "trackers";

/// The final product of user that will go into Database.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HikeTrackerModel {
    pub _id: ObjectId,
    pub name: String,
    pub created_by_id: ObjectId,
    pub hikes: Vec<HikeModel>,
    pub created_at: mongodb::bson::DateTime,
    pub updated_at: mongodb::bson::DateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HikeModel {
    pub name: String,
    pub rank: u8,
    pub created_at: mongodb::bson::DateTime,
    pub updated_at: mongodb::bson::DateTime,
}

pub fn get_tracker_collection(client: &Client) -> Collection<HikeTrackerModel> {
    client
        .database(MONGO_DB_NAME)
        .collection(MONGO_COLL_NAME_TRACKERS)
}

pub async fn init_mongo(uri: &str) -> Result<mongodb::Client> {
    let client = mongodb::Client::with_uri_str(uri).await?;

    let options = mongodb::options::IndexOptions::builder()
        .unique(true)
        .build();

    let hike_tracker_model = mongodb::IndexModel::builder()
        .keys(mongodb::bson::doc! { "name": 1 })
        .options(options.clone())
        .build();

    client
        .database(MONGO_DB_NAME)
        .collection::<HikeTrackerModel>(MONGO_COLL_NAME_TRACKERS)
        .create_index(hike_tracker_model, None)
        .await?;

    Ok(client)
}

pub async fn seed_data(client: &Client) -> Result<()> {
    let tracker = HikeTrackerModel {
        _id: ObjectId::new(),
        name: "first".to_string(),
        created_by_id: ObjectId::new(),
        hikes: vec![],
        created_at: chrono::Utc::now().into(),
        updated_at: chrono::Utc::now().into(),
    };

    let found = get_tracker_collection(&client)
        .find_one(bson::doc! { "name": "first" }, None)
        .await?;

    if found.is_none() {
        get_tracker_collection(&client)
            .insert_one(tracker, None)
            .await?;
    }

    Ok(())
}
