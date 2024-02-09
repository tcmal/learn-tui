use serde::Deserialize;

use crate::{course::Course, Client, Result};

/// Ties a user to a course
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMembership {
    pub id: String,
    pub user_id: String,
    pub course_id: String,
    pub course: Course,
}

#[derive(Deserialize)]
struct UserMembershipResp {
    results: Vec<UserMembership>,
}

impl Client {
    pub fn user_memberships(&self, user_id: &str) -> Result<Vec<UserMembership>> {
        self.get::<UserMembershipResp>(&format!(
            "learn/api/public/v1/users/{}/courses?expand=course",
            user_id
        ))
        .map(|r| r.results)
    }
}
