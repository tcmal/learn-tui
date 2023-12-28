use std::collections::HashSet;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use tui_tree_widget::{Tree, TreeItem, TreeState};

use super::{content, courses, Action, ActivePage, Page};
use crate::store::{ContentIdx, Store};

pub struct State {
    course_id: String,
    loading: HashSet<ContentIdx>,
    tree_state: TreeState<ContentIdx>,
    last_items: Vec<TreeItem<'static, ContentIdx>>,
}
impl State {
    pub(crate) fn new(course_id: String) -> State {
        Self {
            course_id,
            tree_state: Default::default(),
            loading: Default::default(),
            last_items: vec![],
        }
    }

    fn rebuild_tree_items(&mut self, store: &Store) {
        let Some(content_idx) = store.course_content(&self.course_id) else {
            self.last_items = vec![TreeItem::new_leaf(0, "Loading...")];
            return;
        };
        self.last_items = vec![self.build_content_subtree(store, content_idx)];
        log::debug!("tree items: {:?}", &self.last_items);
    }
    fn build_content_subtree(
        &mut self,
        store: &Store,
        content_idx: ContentIdx,
    ) -> TreeItem<'static, ContentIdx> {
        let content = store.content(content_idx);
        if !content.has_children.unwrap_or(false) {
            // Leaf node
            return TreeItem::new_leaf(content_idx, content.title.to_string());
        } else if store.content_children_loaded(content_idx) {
            let children = store.content_children(content_idx).unwrap();
            self.loading.remove(&content_idx);
            // Show loaded children
            TreeItem::new(
                content_idx,
                content.title.to_string(),
                children
                    .map(|c| self.build_content_subtree(store, c))
                    .collect(),
            )
            .unwrap()
        } else if self.loading.contains(&content_idx) {
            // Indicate loading
            TreeItem::new_leaf(content_idx, format!("{} - Loading...", content.title))
        } else {
            // Has unloaded children
            TreeItem::new(content_idx, content.title.to_string(), vec![]).unwrap()
        }
    }
}

impl Page for State {
    fn draw(&mut self, store: &Store, frame: &mut Frame) {
        self.rebuild_tree_items(store);
        frame.render_stateful_widget(
            Tree::new(self.last_items.clone())
                .unwrap()
                .highlight_symbol(">>"),
            frame.size(),
            &mut self.tree_state,
        );
    }

    fn handle_key(&mut self, store: &Store, key: KeyEvent) -> Result<Action> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                return Ok(Action::NewScreen(ActivePage::Courses(
                    courses::State::default(),
                )));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.tree_state.key_down(&self.last_items);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.tree_state.key_up(&self.last_items);
            }
            KeyCode::Enter | KeyCode::Tab => {
                let sel = self.tree_state.selected();
                if let Some(&content_idx) = sel.last() {
                    let content = store.content(content_idx);
                    if !content.has_children.unwrap_or(false) {
                        return Ok(Action::NewScreen(ActivePage::Content(content::State::new(
                            self.course_id.clone(),
                            content_idx,
                        ))));
                    }

                    if let None = store.content_children(content_idx) {
                        // requesting content children queues it up
                        self.loading.insert(content_idx);
                    } else {
                        self.tree_state.toggle(sel);
                    }
                }
            }
            _ => (),
        };

        Ok(Action::None)
    }
}
