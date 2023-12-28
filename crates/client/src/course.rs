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
