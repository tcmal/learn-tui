use serde::Deserialize;

use crate::{content::Content, Client, Result};

#[derive(Debug, Deserialize)]
pub struct Course {
    pub id: String,
    pub uuid: String,
    #[serde(rename = "courseId")]
    pub course_id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "termId")]
    pub term_id: Option<String>,
}

impl Client {
    pub fn course_content(&self, course_id: &str) -> Result<Content> {
        self.get::<Content>(&format!(
            "learn/api/public/v1/courses/{}/contents/ROOT",
            course_id
        ))
        .map(|mut r| {
            r.course_id = course_id.to_string();
            r
        })
    }
}
