use serde::Deserialize;

use crate::{Client, Result};

#[derive(Debug, Deserialize)]
pub struct Term {
    pub name: String,
    pub id: String,
    // "isAvailable": true,
    // "endDate": null,
    // "startDate": null,
    // "durationType": "CONTINUOUS",
    // "daysOfUse": 0,
    // "description": {
    // 	"rawText": "<p>2023-2024 [SEM1]</p>",
    // 	"displayText": "<div class=\"vtbegenerated\">\n <p>2023-2024 [SEM1]</p>\n</div>",
    // 	"webLocation": null,
    // 	"fileLocation": null
    // },
    // "permissions": {
    // 	"delete": false,
    // 	"edit": false
    // },
    // "name": "2023-2024 [SEM1]",
    // "id": "_507_1"
}

#[derive(Deserialize)]
struct RawResp {
    results: Vec<Term>,
}

impl Client {
    pub fn terms(&self) -> Result<Vec<Term>> {
        Ok(self.get::<RawResp>("learn/api/v1/terms")?.results)
    }
}
