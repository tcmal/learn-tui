use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;

use crate::{Client, Error, Result};

impl Client {
    pub fn content_children(&self, course_id: &str, content_id: &str) -> Result<Vec<Content>> {
        Ok(self
            .get::<ContentChildrenResp>(&format!(
                "learn/api/public/v1/courses/{}/contents/{}/children",
                course_id, content_id
            ))?
            .results
            .into_iter()
            .map(|raw| Content::new(raw, course_id))
            .collect())
    }

    pub fn page_text(&self, course_id: &str, content_id: &str) -> Result<String> {
        let mut results = self
            .get::<ContentChildrenResp>(&format!(
                "learn/api/public/v1/courses/{}/contents/{}/children",
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
    pub link: Option<String>,

    pub payload: ContentPayload,
}

impl Content {
    fn new(raw: RawContent, course_id: &str) -> Self {
        let payload = match raw
            .content_detail
            .filter(|d| !matches!(d, ContentDetail::Unknown))
            .or(raw.content_handler)
        {
            Some(ContentDetail::ExternalLink { url }) => ContentPayload::Link(url),
            Some(ContentDetail::Folder { is_page: true }) => ContentPayload::Page,
            Some(ContentDetail::Folder { is_page: false }) | Some(ContentDetail::Lesson {}) => {
                ContentPayload::Folder
            }
            Some(ContentDetail::Other(s)) => ContentPayload::Other(s),
            None => ContentPayload::Other("resource/x-bb-api-is-shit".to_string()),
            Some(ContentDetail::Unknown) => unreachable!(), // filter arm above
        };

        Content {
            id: raw.id,
            course_id: course_id.to_string(),
            title: raw.title,
            description: raw.description,
            link: raw.body.and_then(|b| b.web_location), // TODO: sometimes there's a link attribute you can get this out of if theres no body, need to investigate more
            payload,
        }
    }

    pub fn is_container(&self) -> bool {
        matches!(self.payload, ContentPayload::Folder)
    }
}

/// What the content is, and the actual content if it carries it.
#[derive(Debug)]
pub enum ContentPayload {
    /// A link to some website.
    Link(String),

    /// A folder, with more content inside.
    Folder,

    /// A page. Use [`Self::page_contents`] to get the actual text.
    Page,

    /// Something else. The contained string is the content handler, which might be a hint.
    Other(String),
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
struct RawContent {
    id: String,

    title: String,
    description: Option<String>,

    // sometimes this is just a string for raw_text!
    #[serde(deserialize_with = "raw_body_str_or_struct", default = "none")]
    body: Option<RawContentBody>,
    #[serde(rename = "contentDetail")]
    content_detail: Option<ContentDetail>,
    // sometimes this just contains the data that would be in content_detail in a different format! fun!
    #[serde(rename = "contentHandler", deserialize_with = "handler_to_detail")]
    content_handler: Option<ContentDetail>,
}

#[derive(Debug, Deserialize)]
struct RawContentBody {
    #[serde(rename = "rawText")]
    raw_text: String,
    #[serde(rename = "webLocation")]
    web_location: Option<String>,
}

#[derive(Debug, Deserialize)]
enum ContentDetail {
    #[serde(rename = "resource/x-bb-externallink")]
    ExternalLink { url: String },

    #[serde(rename = "resource/x-bb-folder")]
    Folder { is_page: bool },

    #[serde(rename = "resource/x-bb-lesson")]
    Lesson,

    #[serde(skip)]
    Other(String),

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
struct RawContentHandler {
    id: String,
    url: Option<String>,
    #[serde(rename = "isBbPage")]
    is_bb_page: Option<bool>,
}

fn handler_to_detail<'de, D>(deserializer: D) -> Result<Option<ContentDetail>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrStruct;

    impl<'de> Visitor<'de> for StringOrStruct {
        type Value = ContentDetail;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(ContentDetail::Other(v.to_string()))
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let raw: RawContentHandler =
                Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))?;

            match raw {
                RawContentHandler {
                    id, url: Some(url), ..
                } if id == "resource/x-bb-externallink" => Ok(ContentDetail::ExternalLink { url }),
                RawContentHandler { id, is_bb_page, .. } if id == "resource/x-bb-folder" => {
                    Ok(ContentDetail::Folder {
                        is_page: is_bb_page.unwrap_or(false),
                    })
                }
                RawContentHandler { id, .. } if id == "resource/x-bb-lesson" => {
                    Ok(ContentDetail::Lesson)
                }
                RawContentHandler { id, .. } => Ok(ContentDetail::Other(id)),
            }
        }
    }

    match deserializer.deserialize_any(StringOrStruct) {
        Ok(v) => Ok(Some(v)),
        Err(_) => Ok(None),
    }
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
                web_location: None,
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
