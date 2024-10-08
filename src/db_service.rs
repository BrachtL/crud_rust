use std::{collections::HashMap, fmt};

use firestore::{FirestoreDb, FirestoreDbOptions, errors::FirestoreError};
use log::info;
use serde_json::{Map, Value};
use crate::model::User;
use uuid::Uuid; // To generate a temporary UUID for the user

#[derive(Debug)]
pub enum MyError {
    FirestoreError(FirestoreError),
    UserNotFound(String),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::FirestoreError(err) => write!(f, "Firestore error: {}", err),
            MyError::UserNotFound(id) => write!(f, "User not found with id: {}", id),
        }
    }
}

// Optionally, you can also implement std::error::Error for MyError
impl std::error::Error for MyError {}

pub struct DbService {
    client: FirestoreDb,
}

impl DbService {
    pub async fn new() -> Result<DbService, FirestoreError> {
        let client = FirestoreDb::with_options_service_account_key_file(
            FirestoreDbOptions::new(String::from("crud1-ebc0e")),
            "./crud1-ebc0e-firebase-adminsdk-ci2ex-e80a1dadda.json".into()
        ).await?;

        Ok(DbService { client })
    }

    pub async fn get_all(&self) -> Result<Vec<User>, FirestoreError> {
        let users: Vec<User> = self.client
            .fluent()
            .select()
            .from("Users")
            .obj()
            .query()
            .await?;

        Ok(users)
    }

    // Insert function that generates a Firestore ID and adds it to the user data
    pub async fn insert(&self, mut user: User) -> Result<String, FirestoreError> {
        //user.id = Uuid::new_v4().to_string();
        // Generate the document ID using Firestore's auto-generation feature
        let doc_ref = self.client
            .fluent()
            .insert()
            .into("Users")
            .document_id(&user.id) // Let Firestore generate the ID
            .object(&user) // Pass the user object to Firestore
            .execute::<User>() // Specify that we're inserting a `User`
            .await?;


        //log::info!("doc_ref -> {:?}", doc_ref.id);

        Ok(user.id) // Return the generated document ID
    }

    // Update a user by their ID
    pub async fn update_by_id(&self, user_id: String, user_data: Map<String, Value>) -> Result<(), MyError> {
        self.client
            .fluent()
            .update()
            .fields(&user_data.keys().cloned().collect::<Vec<String>>()) // Pass the valid field names
            .in_col("Users") // Specify the collection
            .document_id(&user_id) // Use the Firestore document ID of the found document
            .object(&user_data) // Pass the updates object
            .execute::<User>() // Specify that we're updating a `User`
            .await
            .map_err(MyError::FirestoreError)?; // Map FirestoreError to MyError
    
        Ok(())
    }

    // Delete a user by ID
    pub async fn delete_by_id(&self, id: String) -> Result<(), FirestoreError> {
        self.client
            .fluent()
            .delete()
            .from("Users")
            .document_id(&id)
            .execute()
            .await?;

        Ok(())
    }

    pub async fn get_user_by_id(&self, id: String) -> Result<User, MyError> {
        let user: Option<User> = self.client
            .fluent()
            .select()
            .by_id_in("Users") // Query the document by its ID
            .obj()
            .one(&id)
            .await
            .map_err(MyError::FirestoreError)?; // Convert FirestoreError to MyError
    
        // Convert Option<User> to Result<User, FirestoreError>
        user.ok_or_else(|| MyError::UserNotFound(id))
    }

    pub async fn check_user_exists(&self, user_id: String) -> Result<bool, MyError> {
        let documents: Vec<User> = self.client
            .fluent()
            .select()
            .from("Users")
            .filter(|filter| filter.field("id").equal(&user_id)) // Use the correct method for filtering
            .obj()
            .query()
            .await
            .map_err(MyError::FirestoreError)?;
    
        // Return true if any user is found, false otherwise
        Ok(!documents.is_empty())
    }

}


