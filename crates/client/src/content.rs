use chrono::{DateTime, Local};
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;

use crate::{Client, Error, Result, LEARN_BASE};

impl Client {
    /// Get the top-level children of a course
    pub fn course_children(&self, course_id: &str) -> Result<Vec<Content>> {
        self.content_children(course_id, "ROOT")
    }

    /// Get the children of a given content item.
    pub fn content_children(&self, course_id: &str, content_id: &str) -> Result<Vec<Content>> {
        Ok(self
            .get::<ContentChildrenResp>(&format!(
                "learn/api/v1/courses/{}/contents/{}/children",
                course_id, content_id
            ))?
            .results
            .into_iter()
            .map(|raw| Content::new(raw, course_id))
            .collect())
    }

    /// Get the text of a page
    pub fn page_text(&self, course_id: &str, content_id: &str) -> Result<String> {
        let mut results = self
            .get::<ContentChildrenResp>(&format!(
                "learn/api/v1/courses/{}/contents/{}/children",
                course_id, content_id
            ))?
            .results;
        if results.len() != 1 {
            return Err(Error::BadContentLeaf);
        }

        let result = results.pop().unwrap();
        let Some(RawContentBody { raw_text, .. }) = result.body else {
            return Err(Error::BadContentLeaf);
        };

        Ok(raw_text)
    }
}

/// A piece of content, heavily edited to have some structure.
/// These act like directory trees within a course.
#[derive(Debug)]
pub struct Content {
    pub id: String,
    pub course_id: String,

    pub title: String,
    pub description: Option<String>,

    pub payload: ContentPayload,

    link: String,
}

impl Content {
    fn new(raw: RawContent, course_id: &str) -> Self {
        let payload = match raw.content_detail {
            Some(ContentDetail::ExternalLink { url }) => ContentPayload::Link(url),
            Some(ContentDetail::Folder { is_page: true }) => ContentPayload::Page,
            Some(ContentDetail::Folder { is_page: false }) | Some(ContentDetail::Lesson {}) => {
                ContentPayload::Folder
            }
            Some(ContentDetail::File {
                file:
                    RawFile {
                        mime_type,
                        permanent_url,
                        file_name,
                    },
            }) => ContentPayload::File {
                file_name,
                mime_type,
                permanent_url: format!(
                    "{}{}",
                    LEARN_BASE,
                    permanent_url.strip_prefix('/').unwrap()
                ),
            },
            // The returned URL is relative to the learn base, and is normally broken and shows the old learn interface nested a bunch of times
            // This is fixed by adding `&from_ultra=true`, as learn ultra does.
            Some(ContentDetail::Piazza { launch_link }) => ContentPayload::Placement {
                name: "Piazza",
                url: format!("{}{}&from_ultra=true", LEARN_BASE, launch_link),
            },
            Some(ContentDetail::MediaHopperReplay { launch_link }) => ContentPayload::Placement {
                name: "Media Hopper Replay",
                url: format!("{}{}&from_ultra=true", LEARN_BASE, launch_link),
            },
            Some(ContentDetail::Zoom { launch_link }) => ContentPayload::Placement {
                name: "Zoom",
                url: format!("{}{}&from_ultra=true", LEARN_BASE, launch_link),
            },
            Some(ContentDetail::Assessment { test }) => ContentPayload::Assessment {
                name: test.grading_column.effective_column_name,
                due_date: test.grading_column.due_date,
            },
            Some(ContentDetail::Unknown {}) | None => ContentPayload::Other,
        };

        Content {
            link: format!(
                "{}ultra/redirect?redirectType=nautilus&courseId={}&contentId={}&parentId={}",
                LEARN_BASE, course_id, raw.id, raw.parent_id
            ),
            id: raw.id,
            course_id: course_id.to_string(),
            title: raw.title,
            description: raw.description,
            payload,
        }
    }

    pub fn is_container(&self) -> bool {
        matches!(self.payload, ContentPayload::Folder)
    }

    pub fn browser_link(&self) -> &str {
        match &self.payload {
            ContentPayload::Link(link) => link,
            ContentPayload::File { permanent_url, .. } => permanent_url,
            ContentPayload::Placement { url, .. } => url,
            _ => &self.link,
        }
    }
}

/// What the content is, and the actual content if it carries it.
#[derive(Debug)]
pub enum ContentPayload {
    /// A link to some website.
    Link(String),

    /// A folder, with more content inside.
    Folder,

    /// A page. Use [`Client::page_text`] to get the actual text.
    Page,

    /// Something else.
    Other,

    /// A file, may meant to be downloaded or embedded.
    File {
        mime_type: String,
        file_name: String,
        permanent_url: String,
    },

    /// Link to a placement in some other application.
    /// URL will authenticate and then redirect the user.
    Placement { name: &'static str, url: String },

    Assessment {
        name: String,
        due_date: DateTime<Local>,
    },
}

#[derive(Deserialize)]
pub struct ContentChildrenResp {
    results: Vec<RawContent>,
}

// so firstly, everything on the blackboard learn api docs site is a lie.
// content items actually seem to follow this pattern:
//   - for folders, we get ContentDetail::Folder, with is_page set to false
//     'lessons' have a different name but seem to basically be folders
//      with special display options that we ignore
//   - for pages, we get ContentDetail::Folder, with is_page set to true
//     if you query its child, you get what im calling a 'content leaf'
//     content leaves don't have content_detail, just body.
//   - for links, we get ContentDetail::ExternalLink
//   - for other stuff, we get different content details, etc.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawContent {
    id: String,
    parent_id: String,

    title: String,
    description: Option<String>,

    // sometimes this is just a string for raw_text!
    #[serde(deserialize_with = "raw_body_str_or_struct", default = "none")]
    body: Option<RawContentBody>,
    content_detail: Option<ContentDetail>,
}

#[derive(Debug, Deserialize)]
struct RawContentBody {
    #[serde(rename = "rawText")]
    raw_text: String,
}

#[derive(Debug, Deserialize)]
enum ContentDetail {
    #[serde(rename = "resource/x-bb-externallink")]
    ExternalLink { url: String },

    #[serde(rename = "resource/x-bb-folder")]
    Folder {
        #[serde(rename = "isBbPage", default = "val_false")]
        is_page: bool,
    },

    #[serde(rename = "resource/x-bb-lesson")]
    Lesson {},

    #[serde(rename = "resource/x-bb-file")]
    File { file: RawFile },

    #[serde(rename = "resource/x-bb-bltiplacement-49f1179af0494f078ce3ff737dd75de4")]
    #[serde(rename_all = "camelCase")]
    Piazza { launch_link: String },

    #[serde(rename = "resource/x-bb-bltiplacement-mhrlti")]
    #[serde(rename_all = "camelCase")]
    MediaHopperReplay { launch_link: String },

    #[serde(rename = "resource/x-bb-bltiplacement-zoom")]
    #[serde(rename_all = "camelCase")]
    Zoom { launch_link: String },

    #[serde(rename = "resource/x-bb-asmt-test-link")]
    #[serde(rename_all = "camelCase")]
    Assessment { test: RawTest },

    #[serde(untagged)]
    Unknown {},
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawFile {
    mime_type: String,
    file_name: String,
    permanent_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawTest {
    grading_column: RawGradingColumn,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawGradingColumn {
    effective_column_name: String,
    due_date: DateTime<Local>,
}

fn raw_body_str_or_struct<'de, D>(deserializer: D) -> Result<Option<RawContentBody>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrStruct;

    impl<'de> Visitor<'de> for StringOrStruct {
        type Value = RawContentBody;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(RawContentBody {
                raw_text: v.to_string(),
            })
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    match deserializer.deserialize_any(StringOrStruct) {
        Ok(v) => Ok(Some(v)),
        Err(_) => Ok(None),
    }
}
fn none<T>() -> Option<T> {
    None
}

fn val_false() -> bool {
    false
}
