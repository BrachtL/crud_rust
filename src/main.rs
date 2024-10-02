// main.rs

mod model;
mod db_service;
mod routes; // Import the routes module
mod errors;

use db_service::DbService;
use std::sync::Arc;
use tokio::sync::Mutex;
//use warp::Filter;

#[tokio::main]
async fn main() {
    env_logger::init();
    // Initialize the Firestore client
    let db_service = Arc::new(Mutex::new(DbService::new().await.unwrap()));

    // Create the routes using the new routes module
    let routes = routes::create_routes(db_service.clone());

    // Start the Warp server
    println!("Listening on 0.0.0.0:3000");
    warp::serve(routes).run(([0, 0, 0, 0], 3000)).await;
}