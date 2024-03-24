use anyhow::Result;
use maud::{html, Markup, DOCTYPE};
use std::{fs::read_to_string, ops::Deref, sync::Arc};

use askama::Template;
use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Extension, Router,
};
use mongo::{HikeTrackerModel, MONGO_COLL_NAME_TRACKERS, MONGO_DB_NAME};
use mongodb::{Client, Collection};
use serde::{Deserialize, Serialize};

use crate::mongo::{get_tracker_collection, init_mongo, seed_data};

mod mongo;

#[derive(Debug, Serialize, Deserialize)]
pub struct HikePeak {
    name: String,
    elevation: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    pub const DEFAULT_MONGO_URI: &str = "mongodb://0.0.0.0:27017";

    let client: Arc<Client> = Arc::new(init_mongo(DEFAULT_MONGO_URI).await?);

    seed_data(&client).await?;

    let available_hikes: Arc<Vec<HikePeak>> =
        Arc::new(serde_json::from_str(&read_to_string("./hike_peaks.json").unwrap()).unwrap());

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/peaks", get(hike_peak_list))
        .route("/tracker/:id", get(display_tracker))
        .route("/tracker/:id/edit", get(edit_tracker))
        .layer(Extension(available_hikes))
        .route("/clicked", post(clicked))
        .layer(Extension(client));

    // run our app with hyper, listening globally on port 3000
    let host_port = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(host_port).await.unwrap();
    println!("Listening on: {host_port}");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn hike_peak_list(Extension(hikes): Extension<Arc<Vec<HikePeak>>>) -> Markup {
    let hikes = hikes.iter();
    html! {
        @for hike in hikes {
            li { "name: "(hike.name)", elevation: "(hike.elevation) }
        }
    }
}

async fn display_tracker(
    Path(id): Path<String>,
    Extension(client): Extension<Arc<Client>>,
) -> Markup {
    if let Some(t) = get_tracker_collection(&client)
        .find_one(bson::doc! { "name": id.clone() }, None)
        .await
        .unwrap()
    {
        boiler(html! {
            div hx-target="this" hx-swap="outerHTML" {
                div { label { "Name: " } (t.name) }
                button hx-get={ "/tracker/"(id)"/edit" } class="btn btn-primary" { "Click To Edit" }
            }
        })
    } else {
        html! { h1 { "Not Found" } }
    }
}

async fn edit_tracker(Path(id): Path<String>, Extension(client): Extension<Arc<Client>>) -> Markup {
    if let Some(t) = get_tracker_collection(&client)
        .find_one(bson::doc! { "name": id.clone() }, None)
        .await
        .unwrap()
    {
        html! {
            form hx-put="/(id)" hx-target="this" hx-swap="outerHTML" {
                div {
                    label { "Name" }
                    input type="text" name="trackerName" value=(t.name);
                }
                button class="btn" { "Submit" }
                button class="btn" hx-get={ "/tracker/"(id) } { "Cancel" }
            }
        }
    } else {
        html! { h1 { "Not Found" } }
    }
}

fn boiler(body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
              meta charset="utf-8";
              meta name="viewport" content="width=device-width, initial-scale=1";
              title { "TEST" }
              script src="https://unpkg.com/htmx.org@1.9.11" integrity="sha384-0gxUXCCR8yv9FM2b+U3FDbsKthCI66oH5IA9fHppQq9DDMHuMauqq1ZHBpJxQ0J0" crossorigin="anonymous" {}
            }
            body {
                h1 { "The Body Mint" }
                div id="the-one" {
                  "Div"
                }
                (body)
            }
        }
    }
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
