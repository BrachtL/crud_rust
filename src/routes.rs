// routes.rs

use crate::{db_service::MyError, model::User};
use crate::db_service::DbService;
use log::info;
use serde_json::{json, Map, Value};
use warp::reject::Reject;
use warp::reply::Json;
use std::convert::Infallible;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use warp::{reject, Filter, Rejection, Reply};

#[derive(Debug)]
struct MissingIdError;

#[derive(Debug)]
struct UserNotFoundError;
impl Reject for UserNotFoundError {}

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
) -> Result<impl warp::Reply, Rejection> {
    let db_service = db_service.lock().await;

    // Extract the user ID from the updates Value
    let user_id = updates
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| warp::reject::custom(MissingIdError))?;

    info!("user_id -> {}", user_id);

    // Check if the user exists in the database
    let user_exists = match db_service.check_user_exists(user_id.to_string()).await {
        Ok(exists) => exists,
        Err(_) => return Err(warp::reject::custom(UserNotFoundError)),
    };

    if !user_exists {
        return Err(warp::reject::custom(UserNotFoundError)); // Return custom error if the user is not found
    }

    // Convert updates Value to Map<String, Value> before passing to update_by_id
    let update_map: Map<String, Value> = updates
        .as_object()
        .unwrap_or(&Default::default()) // Ensure we have an object
        .iter()
        .filter_map(|(key, value)| {
            if !value.is_null() {
                Some((key.clone(), value.clone())) // Only add non-null fields
            } else {
                None
            }
        })
        .collect();

    info!("update_fields -> {:?}", update_map);

    // Perform the update if the user exists
    let update_result = db_service.update_by_id(user_id.to_string(), update_map).await;
    
    match update_result {
        Ok(_) => Ok(warp::reply::json(&json!({
            "status": "User updated successfully"
        }))),
        Err(_) => Err(warp::reject::custom(UserNotFoundError)), // Handle errors during the update
    }
}

// Handler for deleting a user by ID
pub async fn handle_delete_user(
    id: String,
    db_service: Arc<Mutex<DbService>>,
) -> Result<impl Reply, Rejection> {
    let db_service = db_service.lock().await;

    // Check if the user exists
    let user_exists = db_service
        .check_user_exists(id.clone()) // Use the user ID to check existence
        .await
        .map_err(|_| warp::reject::custom(UserNotFoundError))?; // Reject with custom error if not found

    if !user_exists {
        return Err(warp::reject::custom(UserNotFoundError)); // Return custom rejection if not found
    }

    // Proceed with deletion if user exists
    match db_service.delete_by_id(id.clone()).await {
        Ok(_) => Ok(warp::reply::json(&json!({
            "status": "User deleted successfully"
        }))),
        Err(e) => Ok(warp::reply::json(&json!({
            "status": "Error deleting user",
            "error": e.to_string()
        }))),
    }
}

pub async fn handle_get_all_users(
    db_service: Arc<Mutex<DbService>>,
) -> Result<impl warp::Reply, Rejection> {
    let db_service = db_service.lock().await;

    match db_service.get_all().await {
        Ok(users) => {
            Ok(warp::reply::json(&json!( {
                "status": "Success",
                "users": users
            })))
        }
        Err(e) => Ok(warp::reply::json(&json!( {
            "status": "Error fetching users",
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

    // New route for getting all users
    let get_all_users = warp::get()
        .and(warp::path("get_all_users"))
        .and(with_db_service(db_service.clone())) // Pass the DB service
        .and_then(handle_get_all_users);

    insert_user.or(update_user).or(delete_user).or(get_all_users) // Combine the filters
}

/* 
async fn handle_rejection(err: Rejection) -> Result<Json, Infallible> {
    if let Some(_) = err.find::<UserNotFoundError>() {
        // Return a JSON response when UserNotFoundError is encountered
        let json_error = json!({
            "status": "error",
            "message": "User not found"
        });
        return Ok(warp::reply::json(&json_error));
    }

    // Fallback for any other rejections (e.g., unhandled errors)
    let json_error = json!({
        "status": "error",
        "message": "An unknown error occurred"
    });
    Ok(warp::reply::json(&json_error))
}
*/