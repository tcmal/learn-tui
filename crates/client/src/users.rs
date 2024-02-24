use std::collections::HashMap;

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

    /// Get the current user's favourite courses.
    /// Returns a list of course IDs
    pub fn my_favourites(&self) -> Result<Vec<String>> {
        let resp: FavCoursesResp =
            self.get("learn/api/v1/users/me/preferences/favorite.courses")?;
        let inner: FavCoursesInner = serde_json::from_str(&resp.value)?;

        Ok(inner
            .into_iter()
            .filter(|(_, v)| *v)
            .map(|(k, _)| k)
            .collect())
    }
}

#[derive(Debug, Deserialize)]
struct FavCoursesResp {
    value: String,
}

type FavCoursesInner = HashMap<String, bool>;
