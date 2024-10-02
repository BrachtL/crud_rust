/* 
#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, World!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
*/



// main.rs

mod model;
mod db_service;

use model::User;
use db_service::DbService;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;


#[tokio::main]
async fn main() {
    env_logger::init();
    // Initialize the Firestore client
    let db_service = Arc::new(Mutex::new(DbService::new().await.unwrap()));

    // Define the warp filter
    let insert_user = warp::post()
        .and(warp::path("insert_user"))
        .and(warp::body::json()) // Automatically deserialize JSON body to User
        .and(with_db_service(db_service.clone())) // Pass the DB service
        .and_then(handle_insert_user);

    // Start the Warp server
    let routes = insert_user;

    println!("Listening on 0.0.0.0:3000");
    warp::serve(routes).run(([0, 0, 0, 0], 3000)).await;
}

// Custom filter to pass the DB service
fn with_db_service(
    db_service: Arc<Mutex<DbService>>,
) -> impl Filter<Extract = (Arc<Mutex<DbService>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db_service.clone())
}

// Handler for inserting a user and returning the generated ID to the client
async fn handle_insert_user(
    user: User,
    db_service: Arc<Mutex<DbService>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let db_service = db_service.lock().await;

    match db_service.insert(user).await {
        Ok(generated_id) => {
            // Send the generated ID back to the client
            Ok(warp::reply::json(&json!({
                "status": "User inserted successfully",
                "id": generated_id // Include the generated ID in the response
            })))
        }
        Err(e) => Ok(warp::reply::json(&json!({
            "status": "Error inserting user",
            "error": e.to_string()
        }))),
    }
}