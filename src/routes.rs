use crate::db_service::DbService;
use crate::model::User;
use rocket::State;
use uuid::Uuid;
use fake::{faker::name::en::FirstName, faker::internet::en::FreeEmail, faker::phone_number::en::PhoneNumber, Fake};

#[post("/add_user")]
pub async fn add_user(db: &State<DbService>) -> Result<&'static str, String> {
    // Generate a new fake user
    let user = User {
        id: Uuid::new_v4().to_string(),
        first_name: FirstName().fake(),
        email: FreeEmail().fake(),
        phone: PhoneNumber().fake(),
    };

    // Use the DbService to insert the user
    db.insert(user)
        .await
        .map_err(|e| format!("Failed to insert user: {}", e))?;

    Ok("User added successfully")
}