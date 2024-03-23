use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

pub const MONGO_DB_NAME: &str = "hike-tracker";
pub const MONGO_COLL_NAME_TRACKERS: &str = "trackers";

/// The final product of user that will go into Database.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HikeTrackerModel {
    pub _id: ObjectId,
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

pub async fn init_mongo(uri: &str) -> mongodb::Client {
    let client = mongodb::Client::with_uri_str(uri)
        .await
        .expect("failed to connect");

    let options = mongodb::options::IndexOptions::builder()
        .unique(true)
        .build();

    let wotd_model = mongodb::IndexModel::builder()
        .keys(mongodb::bson::doc! { "name": 1 })
        .options(options.clone())
        .build();
    client
        .database(MONGO_DB_NAME)
        .collection::<HikeTrackerModel>(MONGO_COLL_NAME_TRACKERS)
        .create_index(wotd_model, None)
        .await
        .expect("creating an index should succeed");

    client
}
