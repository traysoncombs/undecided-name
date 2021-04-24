use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Response {
    pub id: String,
    pub message: String,
}
