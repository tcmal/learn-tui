use serde::{Deserialize, Serialize};

use crate::{Client, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub uuid: String,

    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "userName")]
    pub user_name: String,

    #[serde(rename = "givenName")]
    pub given_name: String,
    #[serde(rename = "emailAddress")]
    pub email_address: String,
}

impl Client {
    pub fn me(&self) -> Result<User> {
        self.get("learn/api/v1/users/me")
    }
}
