use std::{fs::read_to_string, sync::Arc};

use askama::Template;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Extension, Router,
};
use mongo::{HikeTrackerModel, MONGO_COLL_NAME_TRACKERS, MONGO_DB_NAME};
use mongodb::{bson::oid::ObjectId, Client, Collection};
use serde::{Deserialize, Serialize};

use crate::mongo::init_mongo;

mod mongo;

#[derive(Debug, Serialize, Deserialize)]
pub struct Hike {
    name: String,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();
    pub const DEFAULT_MONGO_URI: &str = "mongodb://0.0.0.0:27017";
    let client: Arc<Client> = Arc::new(init_mongo(DEFAULT_MONGO_URI).await);

    let tracker = HikeTrackerModel {
        _id: ObjectId::new(),
        created_by_id: ObjectId::new(),
        hikes: vec![],
        created_at: chrono::Utc::now().into(),
        updated_at: chrono::Utc::now().into(),
    };

    client
        .database(MONGO_DB_NAME)
        .collection(MONGO_COLL_NAME_TRACKERS)
        .insert_one(tracker, None)
        .await
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap();

    let available_hikes: Vec<Hike> =
        serde_json::from_str(&read_to_string("./hikes.json").unwrap()).unwrap();

    println!("hikes: {available_hikes:?}");

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/clicked", post(clicked))
        .layer(Extension(client));

    // run our app with hyper, listening globally on port 3000
    let host_port = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(host_port).await.unwrap();
    println!("Listening on: {host_port}");
    axum::serve(listener, app).await.unwrap();
}

#[derive(Template)]
#[template(path = "clicked.html")]
struct ClickedTemplate {}

async fn clicked() -> ClickedTemplate {
    ClickedTemplate {}
}

#[derive(Template)]
#[template(path = "base.html")]
struct BaseTemplate<'a> {
    title: &'a str,
}

#[derive(Template)]
#[template(path = "child.html")]
struct ChildTemplate<'a> {
    title: &'a str,
    _parent: &'a BaseTemplate<'a>,
}

// basic handler that responds with a static string
async fn root(
    Extension(client): Extension<std::sync::Arc<Client>>,
) -> Result<ChildTemplate<'static>, StatusCode> {
    let collection: Collection<HikeTrackerModel> = client
        .database(MONGO_DB_NAME)
        .collection(MONGO_COLL_NAME_TRACKERS);

    let tracker = collection
        .find_one(mongodb::bson::doc! { "name": "first" }, None)
        .await
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    println!("tracker: {tracker:?}");

    Ok(ChildTemplate {
        title: "Child Title",
        _parent: &BaseTemplate {
            title: "Parent Title",
        },
    })
}
