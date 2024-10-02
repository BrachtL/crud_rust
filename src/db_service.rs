use firestore::{FirestoreDb, FirestoreDbOptions, errors::FirestoreError};
use crate::model::User;
use uuid::Uuid; // To generate a temporary UUID for the user

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
            .generate_document_id() // Let Firestore generate the ID
            .object(&user) // Pass the user object to Firestore
            .execute::<User>() // Specify that we're inserting a `User`
            .await?;


        //log::info!("doc_ref -> {:?}", doc_ref.id);

        Ok(user.id) // Return the generated document ID
    }

    // Update a user by their ID
    pub async fn update_by_id(&self, user: User) -> Result<(), FirestoreError> {
        self.client
            .fluent()
            .update()
            .fields(["first_name"])
            .in_col("Users")
            .document_id(&user.id) // The ID is now mandatory
            .object(&user)
            .execute::<User>()
            .await?;

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

