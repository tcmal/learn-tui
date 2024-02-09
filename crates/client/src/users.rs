use serde::{Deserialize, Serialize};

use crate::{Client, Result};

/// Information about a user
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// Internal bblearn ID
    pub id: String,

    /// *Another* internal bblearn ID
    pub uuid: String,

    /// An external student ID
    pub student_id: String,

    pub user_name: String,

    /// First name
    pub given_name: String,

    /// Registered email address
    pub email_address: String,
}

impl Client {
    /// Get information about the currently logged in user
    pub fn me(&self) -> Result<User> {
        self.get("learn/api/v1/users/me")
    }
}
