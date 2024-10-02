//model.rs

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub first_name: String,
    pub email: String,
    pub phone: String,
}

//todo: deal with the 2 error I got on db_service regarding not having id on document,
//but there is kind of an id that is the document's name or something :S