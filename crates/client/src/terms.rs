use serde::Deserialize;

use crate::{Client, Result};

/// A term / semester
#[derive(Debug, Deserialize)]
pub struct Term {
    pub name: String,
    pub id: String,
}

#[derive(Deserialize)]
struct RawResp {
    results: Vec<Term>,
}

impl Client {
    /// Get registered terms / semesters
    pub fn terms(&self) -> Result<Vec<Term>> {
        Ok(self.get::<RawResp>("learn/api/v1/terms")?.results)
    }
}
