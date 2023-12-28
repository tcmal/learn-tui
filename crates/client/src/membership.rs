use serde::Deserialize;

use crate::{course::Course, Client, Result};

#[derive(Debug, Deserialize)]
pub struct UserMembership {
    pub id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "courseId")]
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
