use bblearn_api::{content::Content, course::Course, users::User};
use std::{collections::HashMap, ops::Range, sync::mpsc::Sender};

mod worker;
pub use worker::StoreWorker;

pub type CourseId = String;
pub type ContentIdx = usize;

/// Global data store
pub struct Store {
    me: Option<User>,
    my_courses: Option<Vec<Course>>,
    contents: Vec<Content>,
    content_children: HashMap<ContentIdx, Range<ContentIdx>>,
    course_contents: HashMap<CourseId, ContentIdx>,
    worker_channel: Sender<Message>,
}

/// Requests sent to the worker thread
pub enum Message {
    LoadMe,
    Quit,
    LoadMyCourses(String),
    LoadCourseContent(String),
    LoadContentChildren(ContentIdx, CourseId, String),
}

/// Messages received by the app from the worker thread
#[derive(Debug)]
pub enum Event {
    Error(anyhow::Error),
    Me(User),
    MyCourses(Vec<Course>),
    CourseContent(String, Content),
    ContentChildren(ContentIdx, Vec<Content>),
}

impl Store {
    pub fn new(worker_channel: Sender<Message>) -> Self {
        Self {
            worker_channel,
            me: Default::default(),
            my_courses: Default::default(),
            course_contents: Default::default(),
            content_children: Default::default(),
            contents: Default::default(),
        }
    }

    pub fn me(&self) -> Option<&User> {
        let ret = self.me.as_ref();
        if ret.is_none() {
            self.worker_channel.send(Message::LoadMe).unwrap();
        }
        ret
    }

    pub fn my_courses(&self) -> Option<&[Course]> {
        let ret = self.my_courses.as_deref();
        if ret.is_none() {
            if let Some(me) = self.me() {
                self.worker_channel
                    .send(Message::LoadMyCourses(me.id.clone()))
                    .unwrap()
            }
        }
        ret
    }

    pub fn course_content(&self, course_id: &CourseId) -> Option<ContentIdx> {
        let ret = self.course_contents.get(course_id).copied();
        if ret.is_none() {
            self.worker_channel
                .send(Message::LoadCourseContent(course_id.clone()))
                .unwrap();
        }
        ret
    }

    pub fn content(&self, content_idx: ContentIdx) -> &Content {
        &self.contents[content_idx]
    }

    pub fn content_children_loaded(&self, content_idx: ContentIdx) -> bool {
        !self.content(content_idx).has_children.unwrap_or(false)
            || self.content_children.contains_key(&content_idx)
    }

    pub fn content_children(&self, content_idx: ContentIdx) -> Option<Range<ContentIdx>> {
        if !self.content(content_idx).has_children.unwrap_or(false) {
            return Some(0..0);
        }

        let ret = self.content_children.get(&content_idx).cloned();
        if ret.is_none() {
            let content = self.content(content_idx);
            self.worker_channel
                .send(Message::LoadContentChildren(
                    content_idx,
                    content.course_id.clone(),
                    content.id.clone(),
                ))
                .unwrap();
        }
        ret
    }

    pub fn event(&mut self, e: Event) {
        match e {
            Event::Error(e) => panic!("{}", e), // TODO
            Event::Me(m) => self.me = Some(m),
            Event::MyCourses(c) => self.my_courses = Some(c),
            Event::CourseContent(course_id, course_content) => {
                self.course_contents.insert(course_id, self.contents.len());
                self.contents.push(course_content);
            }
            Event::ContentChildren(content_idx, children) => {
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
