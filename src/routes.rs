// routes.rs

use crate::{db_service::MyError, model::User};
use crate::db_service::DbService;
use log::info;
use serde_json::{json, Map, Value};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use warp::{reject, Filter, Rejection, Reply};

#[derive(Debug)]
struct MissingIdError;

impl reject::Reject for MissingIdError {}

// Custom filter to pass the DB service
fn with_db_service(
    db_service: Arc<Mutex<DbService>>,
) -> impl Filter<Extract = (Arc<Mutex<DbService>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db_service.clone())
}

// Handler for inserting a user and returning the generated ID to the client
pub async fn handle_insert_user(
    user: User,
    db_service: Arc<Mutex<DbService>>,
) -> Result<impl warp::Reply, Rejection> {
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

// Handler for updating a user by ID
pub async fn handle_update_user(
    updates: Value,
    db_service: Arc<Mutex<DbService>>,
) -> Result<impl Reply, Rejection> {
    let db_service = db_service.lock().await;

    // Extract the user ID from the updates Value
    let user_id = updates.get("id").and_then(Value::as_str).ok_or_else(|| {
        reject::custom(MissingIdError) // Use custom rejection if ID is missing
    })?;

    info!("user_id -> {}", user_id);

    // Create a HashMap to hold the fields to be updated
    let mut update_fields = HashMap::new();

    // Iterate over the updates to filter out null values and collect valid fields
    for (key, value) in updates.as_object().unwrap_or(&Default::default()) {
        if !value.is_null() {
            update_fields.insert(key.clone(), value.clone());
        }
    }

    info!("update_fields -> {:?}", update_fields);

    // Convert HashMap to serde_json::Map<String, Value>
    let update_map: Map<String, Value> = update_fields.into_iter().collect();

    // Perform the update directly with the filtered fields
    match db_service.update_by_id(user_id.to_string(), update_map).await {
        Ok(_) => Ok(warp::reply::json(&json!( {
            "status": "User updated successfully"
        }))),
        Err(err) => {
            let error_response = match err {
                MyError::FirestoreError(firestore_err) => {
                    let error_message = firestore_err.to_string();
                    json!({
                        "status": "Error updating user",
                        "error": error_message
                    })
                },
                MyError::UserNotFound(msg) => {
                    json!({
                        "status": "User not found",
                        "error": msg
                    })
                },
            };
    
            Ok(warp::reply::json(&error_response))
        }
    }
}

// Handler for deleting a user by ID
pub async fn handle_delete_user(
    id: String,
    db_service: Arc<Mutex<DbService>>,
) -> Result<impl Reply, Rejection> {
    let db_service = db_service.lock().await;

    match db_service.delete_by_id(id).await {
        Ok(_) => Ok(warp::reply::json(&json!({
            "status": "User deleted successfully"
        }))),
        Err(e) => Ok(warp::reply::json(&json!({
            "status": "Error deleting user",
            "error": e.to_string()
        }))),
    }
}

// Function to create the routes
pub fn create_routes(db_service: Arc<Mutex<DbService>>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let insert_user = warp::post()
        .and(warp::path("insert_user"))
        .and(warp::body::json()) // Automatically deserialize JSON body to User
        .and(with_db_service(db_service.clone())) // Pass the DB service
        .and_then(handle_insert_user);

    let update_user = warp::put()
        .and(warp::path("update_user"))
        .and(warp::body::json()) // Automatically deserialize JSON body to User
        .and(with_db_service(db_service.clone())) // Pass the DB service
        .and_then(handle_update_user);

    let delete_user = warp::delete()
        .and(warp::path("delete_user"))
        .and(warp::path::param()) // Extract the user ID from the path
        .and(with_db_service(db_service.clone())) // Pass the DB service
        .and_then(handle_delete_user);

    insert_user.or(update_user).or(delete_user) // Combine the filters
}