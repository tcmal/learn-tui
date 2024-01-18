use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Course {
    pub id: String,
    pub uuid: String,
    #[serde(rename = "courseId")]
    pub course_id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "termId")]
    pub term_id: Option<String>,
    pub created: Option<DateTime<Utc>>,
}
