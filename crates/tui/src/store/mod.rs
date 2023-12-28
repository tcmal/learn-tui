use bblearn_api::{content::Content, course::Course, users::User};
use std::{collections::HashMap, ops::Range, sync::mpsc::Sender};

mod worker;
pub use worker::StoreWorker;

pub type CourseIdx = usize;
pub type ContentIdx = usize;

/// Global data store
pub struct Store {
    me: Option<(User, Vec<Course>)>,

    contents: Vec<Content>,
    content_children: HashMap<ContentIdx, Range<ContentIdx>>,
    course_contents: HashMap<CourseIdx, Range<ContentIdx>>,
    worker_channel: Sender<Message>,
}

/// Requests sent to the worker thread
#[derive(Debug)]
pub enum Message {
    Quit,
    LoadMe,
    LoadCourseContent {
        course_idx: CourseIdx,
        course_id: String,
    },
    LoadContentChildren {
        content_idx: ContentIdx,
        course_id: String,
        content_id: String,
    },
}

/// Messages received by the app from the worker thread
#[derive(Debug)]
pub enum Event {
    Error(anyhow::Error),
    Me(User, Vec<Course>),
    CourseContent {
        course_idx: CourseIdx,
        content: Vec<Content>,
    },
    ContentChildren {
        content_idx: ContentIdx,
        children: Vec<Content>,
    },
}

impl Store {
    pub fn new(worker_channel: Sender<Message>) -> Self {
        Self {
            worker_channel,
            me: Default::default(),
            course_contents: Default::default(),
            content_children: Default::default(),
            contents: Default::default(),
        }
    }

    pub fn my_courses(&self) -> Option<&[Course]> {
        self.me.as_ref().map(|(_, courses)| courses.as_slice())
    }

    pub fn request_my_courses(&self) {
        self.worker_channel.send(Message::LoadMe).unwrap()
    }

    pub fn course_content(&self, course_idx: CourseIdx) -> Option<Range<ContentIdx>> {
        self.course_contents.get(&course_idx).cloned()
    }

    pub fn request_course_content(&self, course_idx: CourseIdx) {
        self.worker_channel
            .send(Message::LoadCourseContent {
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
            .send(Message::LoadContentChildren {
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

    pub fn event(&mut self, e: Event) {
        match e {
            Event::Error(e) => panic!("{}", e), // TODO
            Event::Me(u, cs) => self.me = Some((u, cs)),
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
        }
    }

    pub fn request_quit(&self) {
        self.worker_channel.send(Message::Quit).unwrap();
    }
}
