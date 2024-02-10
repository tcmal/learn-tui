use camino::Utf8PathBuf;
use edlearn_client::{
    content::{Content, ContentPayload},
    course::Course,
    terms::Term,
    users::User,
    Client,
};
use std::{collections::HashMap, ops::Range, sync::mpsc::Sender};

mod downloader;
pub use downloader::Downloader;

mod worker;
pub use worker::Worker;

use crate::{event::EventBus, main_screen::Action, styles::error_text};

pub use self::downloader::{DownloadReq, DownloadState};

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

    download_queue: HashMap<ContentIdx, (DownloadReq, DownloadState)>,

    worker_channel: Sender<Request>,
    downloader_channel: Sender<DownloaderRequest>,
}

/// Requests sent to the worker thread
#[derive(Debug)]
pub(crate) enum Request {
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

#[derive(Debug)]
pub(crate) enum DownloaderRequest {
    DoDownload(ContentIdx, DownloadReq),
}

/// Messages received by the app from the worker or downloader thread
#[derive(Debug)]
pub enum Event {
    Error(edlearn_client::Error),
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
    DownloadState(ContentIdx, DownloadState),
}

impl Store {
    pub fn new(bus: &EventBus, client: Client) -> Self {
        let worker_channel = Worker::spawn_on(bus, client.clone_sharing_state());
        let downloader_channel = Downloader::spawn_on(bus, client);

        Self {
            worker_channel,
            downloader_channel,
            me: Default::default(),
            courses_by_term: Default::default(),
            courses: Default::default(),
            course_contents: Default::default(),
            content_children: Default::default(),
            contents: Default::default(),
            page_texts: Default::default(),
            download_queue: Default::default(),
        }
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

    pub fn download_content(&mut self, content_idx: ContentIdx) {
        let content = self.content(content_idx);
        if let ContentPayload::File {
            file_name,
            permanent_url,
            ..
        } = &content.payload
        {
            // TODO
            let dest = Utf8PathBuf::from(format!("./{}", file_name));
            let req = DownloadReq {
                url: permanent_url.to_string(),
                orig_filename: file_name.to_string(),
                dest,
            };
            self.download_queue
                .insert(content_idx, (req.clone(), DownloadState::Queued));
            self.downloader_channel
                .send(DownloaderRequest::DoDownload(content_idx, req))
                .unwrap();
        }
    }

    /// Get a summary of the current download queue.
    /// Returns (completed, total)
    pub fn download_queue_summary(&self) -> (usize, usize) {
        (
            self.download_queue
                .iter()
                .filter(|(_, (_, state))| matches!(state, DownloadState::Completed))
                .count(),
            self.download_queue.len(),
        )
    }

    pub fn download_queue(&self) -> impl Iterator<Item = &(DownloadReq, DownloadState)> {
        self.download_queue.values()
    }

    pub fn download_status(
        &self,
        content_idx: ContentIdx,
    ) -> Option<&(DownloadReq, DownloadState)> {
        self.download_queue.get(&content_idx)
    }

    pub fn event(&mut self, e: Event) -> Action {
        match e {
            Event::Error(edlearn_client::Error::AuthError(_)) => return Action::Reauthenticate,
            Event::Error(e) => return Action::Flash(error_text(e.to_string())),
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
            Event::DownloadState(r, state) => {
                self.download_queue.entry(r).and_modify(|s| s.1 = state);
            }
        };

        Action::None
    }
}
