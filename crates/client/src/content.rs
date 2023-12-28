use std::fmt;

use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer,
};

use crate::{Client, Result};

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
        todo!()
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
            link: raw.body.map(|b| b.web_location), // TODO: sometimes there's a link attribute you can get this out of if theres no body, need to investigate more
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
    web_location: String,
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
        type Value = Option<ContentDetail>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(ContentDetail::Other(v.to_string())))
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
                } if id == "resource/x-bb-externallink" => {
                    Ok(Some(ContentDetail::ExternalLink { url }))
                }
                RawContentHandler { id, is_bb_page, .. } if id == "resource/x-bb-folder" => {
                    Ok(Some(ContentDetail::Folder {
                        is_page: is_bb_page.unwrap_or(false),
                    }))
                }
                RawContentHandler { id, .. } if id == "resource/x-bb-lesson" => {
                    Ok(Some(ContentDetail::Lesson))
                }
                RawContentHandler { id, .. } => Ok(Some(ContentDetail::Other(id))),
            }
        }
    }

    deserializer.deserialize_any(StringOrStruct)
}
