use chrono::{DateTime, Utc};
use serde::Deserialize;

/// A course
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    /// Internal bbLearn ID
    pub id: String,

    /// Another internal bbLearn ID
    pub uuid: String,

    /// External Course ID
    pub course_id: String,

    pub name: String,
    pub description: Option<String>,
    pub term_id: Option<String>,
    pub created: Option<DateTime<Utc>>,
}
