use firestore::{FirestoreDb, FirestoreDbOptions, errors::FirestoreError};
use crate::model::User;

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

    pub async fn insert(&self, user: User) -> Result<(), FirestoreError> {
      // Insert the user without specifying a document ID
      self.client
          .fluent()
          .insert()
          .into("Users")  // Specify the collection name
          .object(&user)  // The user object to be inserted
          .execute::<()>()
          .await?;
  
      Ok(())
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

    pub async fn update_by_id(&self, user: User) -> Result<(), FirestoreError> {
        self.client
            .fluent()
            .update()
            .fields(["first_name"])
            .in_col("Users")
            .document_id(&user.id)
            .object(&user)
            .execute::<()>()
            .await?;

        Ok(())
    }
}