use std::collections::HashMap;

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
        user.id = Uuid::new_v4().to_string();
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
// Update function to modify user fields by ID
pub async fn update_by_id(&self, user_id: String, user_data: Map<String, Value>) -> Result<(), MyError> {
    // Prepare a HashMap for the updates
    let mut updates = HashMap::new();

    // Iterate through the provided user_data and add to updates, excluding 'id'
    for (key, value) in user_data {
        
            updates.insert(key, value);
        
    }


    self.client
        .fluent()
        .update()
        .fields(&updates.keys().cloned().collect::<Vec<String>>()) // Pass the valid field names
        .in_col("Users") // Specify the collection
        .document_id(&user_id) // Use the Firestore document ID of the found document
        .object(&updates) // Pass the updates object
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
}

