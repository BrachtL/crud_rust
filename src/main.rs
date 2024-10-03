// main.rs

mod model;
mod db_service;
mod routes; // Import the routes module
mod errors;

use db_service::DbService;
use routes::create_routes;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

#[tokio::main]
async fn main() {
    env_logger::init();
    // Initialize the Firestore client
    let db_service = Arc::new(Mutex::new(DbService::new().await.unwrap()));

    let cors = warp::cors()
        .allow_any_origin() // Adjust this in production to only allow specific origins
        .allow_headers(vec![
            "content-type", // Allow Content-Type header
            //"authorization", // Allow Authorization header
            //"accept", // Allow Accept header
            //"origin", // Allow Origin header
            //"referer", // Allow Referer header
            //"user-agent", // Allow User-Agent header
        ])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

    let routes = create_routes(db_service)
        .with(cors);

    // Start the Warp server
    println!("Listening on 0.0.0.0:3000");
    warp::serve(routes).run(([0, 0, 0, 0], 3000)).await;
}