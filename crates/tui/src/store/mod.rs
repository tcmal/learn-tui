use anyhow::Result;
use bblearn_api::{
    content::{Content, ContentPayload},
    course::Course,
    terms::Term,
    users::User,
    Client,
};
use std::{collections::HashMap, ops::Range, sync::mpsc::Sender};

mod worker;
use worker::Worker;

use crate::{event::EventBus, main_screen::Action};

pub type TermIdx = usize;
pub type CourseIdx = usize;
pub type ContentIdx = usize;

/// Global data store
pub struct Store {
    me: Option<User>,

    courses_by_term: Vec<(String, Vec<CourseIdx>)>,
    courses: Vec<Course>,
    contents: Vec<Content>,
    content_children: HashMap<ContentIdx, Range<ContentIdx>>,
    course_contents: HashMap<CourseIdx, Range<ContentIdx>>,

    page_texts: HashMap<ContentIdx, String>,

    worker_channel: Sender<Request>,
}

/// Requests sent to the worker thread
#[derive(Debug)]
enum Request {
    Me,
    CourseContent {
        course_idx: CourseIdx,
        course_id: String,
    },
    ContentChildren {
        content_idx: ContentIdx,
        course_id: String,
        content_id: String,
    },
    PageText {
        content_idx: ContentIdx,
        course_id: String,
        content_id: String,
    },
}

/// Messages received by the app from the worker thread
#[derive(Debug)]
pub enum Event {
    Error(bblearn_api::Error),
    Me(User, Vec<Course>, Vec<Term>),
    CourseContent {
        course_idx: CourseIdx,
        content: Vec<Content>,
    },
    ContentChildren {
        content_idx: ContentIdx,
        children: Vec<Content>,
    },
    PageText {
        content_idx: ContentIdx,
        text: String,
    },
}

impl Store {
    pub fn new(bus: &EventBus, client: Client) -> Result<Self> {
        let worker_channel = Worker::spawn_on(bus, client)?;

        Ok(Self {
            worker_channel,
            me: Default::default(),
            courses_by_term: Default::default(),
            courses: Default::default(),
            course_contents: Default::default(),
            content_children: Default::default(),
            contents: Default::default(),
            page_texts: Default::default(),
        })
    }

    pub fn my_courses(&self) -> Option<&[Course]> {
        self.me.as_ref()?;

        Some(&self.courses)
    }

    pub fn courses_by_term(&self) -> Option<&[(String, Vec<CourseIdx>)]> {
        self.me.as_ref()?;

        Some(&self.courses_by_term)
    }

    pub fn request_my_courses(&self) {
        self.worker_channel.send(Request::Me).unwrap()
    }

    pub fn course_content(&self, course_idx: CourseIdx) -> Option<Range<ContentIdx>> {
        self.course_contents.get(&course_idx).cloned()
    }

    pub fn request_course_content(&self, course_idx: CourseIdx) {
        self.worker_channel
            .send(Request::CourseContent {
                course_idx,
                course_id: self.my_courses().unwrap()[course_idx].id.clone(),
            })
            .unwrap();
    }

    pub fn content_children(&self, content_idx: ContentIdx) -> Option<Range<ContentIdx>> {
        if !self.content(content_idx).is_container() {
            return Some(0..0);
        }

        self.content_children.get(&content_idx).cloned()
    }

    pub fn request_content_children(&self, content_idx: ContentIdx) {
        let content = self.content(content_idx);
        if !content.is_container() {
            return;
        }

        self.worker_channel
            .send(Request::ContentChildren {
                content_idx,
                course_id: content.course_id.clone(),
                content_id: content.id.clone(),
            })
            .unwrap();
    }

    pub fn page_text(&self, content_idx: ContentIdx) -> Option<&str> {
        if !matches!(self.content(content_idx).payload, ContentPayload::Page) {
            return Some("");
        }

        self.page_texts.get(&content_idx).map(|v| v.as_str())
    }

    pub fn request_page_text(&self, content_idx: ContentIdx) {
        let content = self.content(content_idx);
        if !matches!(content.payload, ContentPayload::Page) {
            return;
        }

        self.worker_channel
            .send(Request::PageText {
                content_idx,
                course_id: content.course_id.clone(),
                content_id: content.id.clone(),
            })
            .unwrap();
    }
    pub fn content(&self, content_idx: ContentIdx) -> &Content {
        &self.contents[content_idx]
    }

    pub fn course(&self, course_idx: CourseIdx) -> &Course {
        &self.my_courses().unwrap()[course_idx]
    }

    pub fn event(&mut self, e: Event) -> Action {
        match e {
            Event::Error(bblearn_api::Error::AuthError(_)) => return Action::Reauthenticate,
            Event::Error(e) => panic!("{}", e), // TODO
            Event::Me(u, cs, mut terms) => {
                self.me = Some(u);

                terms.reverse();
                for term in terms {
                    let term_courses = cs
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| c.term_id.as_ref().map(|i| *i == term.id).unwrap_or(false))
                        .map(|(i, _)| i)
                        .collect::<Vec<_>>();

                    if !term_courses.is_empty() {
                        self.courses_by_term.push((term.name, term_courses));
                    }
                }

                self.courses = cs;
            }
            Event::CourseContent {
                course_idx,
                content,
            } => {
                self.course_contents.insert(
                    course_idx,
                    self.contents.len()..self.contents.len() + content.len(),
                );
                self.contents.extend(content);
            }
            Event::ContentChildren {
                content_idx,
                children,
            } => {
                self.content_children.insert(
                    content_idx,
                    self.contents.len()..self.contents.len() + children.len(),
                );
                self.contents.extend(children);
            }
            Event::PageText { content_idx, text } => {
                self.page_texts.insert(content_idx, text);
            }
        };

        Action::None
    }
}
