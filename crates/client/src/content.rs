use serde::Deserialize;

use crate::{Client, Result};

#[derive(Debug, Deserialize)]
pub struct Content {
    /// The ID of the content.
    pub id: String,
    /// The ID of the content's parent.
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,

    /// The title or name of this content. Typically shown as the main text to click in the course outline when accessing the content.
    pub title: String,

    /// The body text associated with this content. This field supports BbML; see <a target='_blank' href='https://docs.anthology.com/docs/rest-apis/learn/advanced/bbml.html'>here</a> for more information.
    #[serde(rename = "body")]
    pub body: Option<String>,
    /// The short description of this content.
    #[serde(rename = "description")]
    pub description: Option<String>,

    /// The date this content was created.
    #[serde(rename = "created")]
    pub created: Option<String>,

    /// The date this content was modified.
    #[serde(rename = "modified")]
    pub modified: Option<String>,

    /// The position of this content within its parent folder. Position values are zero-based (the first element has a position value of zero, not one). Default position is last in the list of child contents under the parent.
    #[serde(rename = "position")]
    pub position: Option<i32>,

    /// Indicates whether this content is allowed to have child content items.
    #[serde(rename = "hasChildren")]
    pub has_children: Option<bool>,
    /// Indicates whether this content item has one or more gradebook columns.  Associated gradebook columns can be accessed via /learn/api/public/v1/courses/$courseId/gradebook/columns?contentId=$contentId  **Since**: 3000.11.0
    #[serde(rename = "hasGradebookColumns")]
    pub has_gradebook_columns: Option<bool>,
    /// Indicates whether this content item has one or more associated groups.  Associated groups can be accessed via /learn/api/public/v1/courses/$courseId/contents/$contentId/groups  **Since**: 3100.4.0
    #[serde(rename = "hasAssociatedGroups")]
    pub has_associated_groups: Option<bool>,

    #[serde(default = "::std::string::String::new")]
    pub course_id: String,
    // #[serde(rename = "availability")]
    // availability: Option<::models::Availability2>,
    // /// Extended settings specific to this content item's content handler.
    // #[serde(rename = "contentHandler")]
    // content_handler: Option<::models::ContentHandler>,
    // /// A list of Content History entities in representation of the copy process the current content item might have if is an LTI content, ordered from newest to oldest content and its respective source course from which current object is a copy of.
    // #[serde(rename = "copyHistory")]
    // copy_history: Option<Vec<::models::ContentCopyHistory>>,
    // /// A list of links to resources related to this content item. Supported relation types include:  - alternate  **Since**: 3900.0.0
    // #[serde(rename = "links")]
    // links: Option<Vec<::models::Link>>,
}

#[derive(Deserialize)]
pub struct ContentChildrenResp {
    results: Vec<Content>,
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

    pub fn content_children(&self, course_id: &str, content_id: &str) -> Result<Vec<Content>> {
        self.get::<ContentChildrenResp>(&format!(
            "learn/api/public/v1/courses/{}/contents/{}/children",
            course_id, content_id
        ))
        .map(|r| r.results)
        .map(|mut cs| {
            for c in cs.iter_mut() {
                c.course_id = course_id.to_string();
            }
            cs
        })
    }
}
