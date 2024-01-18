use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::{prelude::Rect, Frame};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use super::{Action, Document, Pane};
use crate::{event::Event, store::Store};

mod tree;
use tree::*;

/// The navigation pane, which shows a tree structure of all our courses and content
#[derive(Debug, Default)]
pub struct Navigation {
    tree_state: TreeState<TreeId>,
    nav_tree: Vec<NavTree>,
    cached_view_tree: Option<Vec<TreeItem<'static, TreeId>>>,
}

impl Pane for Navigation {
    fn draw(&mut self, store: &Store, frame: &mut Frame, area: Rect) {
        if self.refresh_tree(store) || self.cached_view_tree.is_none() {
            // changed, so refresh view tree
            self.cached_view_tree =
                Some(self.nav_tree.iter().map(|i| i.as_treeitem(store)).collect());
        }

        frame.render_stateful_widget(
            Tree::new(self.cached_view_tree.clone().unwrap())
                .unwrap()
                .highlight_symbol(">>"),
            area,
            &mut self.tree_state,
        );
    }

    fn handle_event(&mut self, store: &Store, event: Event) -> Result<Action> {
        let Event::Key(key) = event else {
            return Ok(Action::None);
        };

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                return Ok(Action::Exit);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.tree_state
                    .key_down(self.cached_view_tree.as_ref().unwrap());
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.tree_state
                    .key_up(self.cached_view_tree.as_ref().unwrap());
            }
            KeyCode::Enter | KeyCode::Tab => {
                let sel = self.tree_state.selected();
                let sel_node = NavTree::navigate_mut(&mut self.nav_tree, &sel);

                match sel_node {
                    // toggle visibility
                    NavTree::Node {
                        children: NavTreeChildren::Done(_),
                        ..
                    } => self.tree_state.toggle(sel),

                    // request loading
                    NavTree::Node {
                        ty,
                        children: children @ NavTreeChildren::NotRequested,
                    } => {
                        ty.request_children(store);
                        *children = NavTreeChildren::Loading;
                        self.tree_state.open(sel);
                        self.cached_view_tree = None;
                    }

                    // show in viewer
                    NavTree::ContentLeaf { content_idx } => {
                        return Ok(Action::Show(Document::Content(*content_idx)));
                    }
                    NavTree::Header {
                        ty: HeaderTy::Welcome,
                    } => {
                        return Ok(Action::Show(Document::Welcome));
                    }

                    // do nothing on loading stuff
                    NavTree::Node {
                        children: NavTreeChildren::Loading,
                        ..
                    } => (),
                    NavTree::Loading => (),
                    NavTree::Header { .. } => (),
                }
            }
            _ => (),
        };

        Ok(Action::None)
    }
}

impl Navigation {
    fn refresh_tree(&mut self, store: &Store) -> bool {
        if self.nav_tree.is_empty() {
            // first call, add courses / loading
            self.nav_tree = vec![NavTree::Loading];
            self.tree_state.select(vec![TreeId::Loading]);
            store.request_my_courses();
            return true;
        }

        let mut changed = false;
        let loading = self.nav_tree.len() == 1 && self.nav_tree[0] == NavTree::Loading;
        if loading {
            if let Some(all_courses) = store.courses_by_term() {
                // done loading
                self.nav_tree.clear();
                self.nav_tree.push(NavTree::Header {
                    ty: HeaderTy::Welcome,
                });
                for (term_idx, (_, courses)) in all_courses.iter().enumerate() {
                    self.nav_tree.push(NavTree::Header {
                        ty: HeaderTy::Term(term_idx),
                    });
                    for course_idx in courses {
                        self.nav_tree.push(NavTree::Node {
                            ty: NodeTy::Course(*course_idx),
                            children: NavTreeChildren::NotRequested,
                        });
                    }
                }

                self.tree_state.select(vec![TreeId::Welcome]);
                changed = true;
            } else {
                // still loading
                return false;
            }
        }

        // loaded/partially loaded tree
        for item in self.nav_tree.iter_mut() {
            if let NavTree::Node {
                ty: NodeTy::Course(course_idx),
                ..
            } = &item
            {
                changed |= Self::refresh_subtree(
                    &mut self.tree_state,
                    store,
                    &mut vec![TreeId::Course(*course_idx)],
                    item,
                );
            }
        }

        changed
    }

    fn refresh_subtree(
        tree_state: &mut TreeState<TreeId>,
        store: &Store,
        id: &mut Vec<TreeId>,
        item: &mut NavTree,
    ) -> bool {
        match item {
            // base case: leaf nodes
            NavTree::Node {
                children: NavTreeChildren::NotRequested,
                ..
            } => false,
            NavTree::ContentLeaf { .. } => false,
            NavTree::Loading => false,
            NavTree::Header { .. } => false,

            // recursively refresh loaded subtrees
            NavTree::Node {
                children: NavTreeChildren::Done(cs),
                ..
            } => cs
                .iter_mut()
                .map(|c| {
                    id.push(c.id());
                    let res = Self::refresh_subtree(tree_state, store, id, c);
                    id.pop();
                    res
                })
                .fold(false, |acc, changed| acc | changed), // non shortcircuiting .any()

            // check if loading subtrees have finished
            NavTree::Node {
                ty,
                children: children @ NavTreeChildren::Loading,
            } => {
                if let Some(new_children) = ty.new_children_loaded(store) {
                    *children = NavTreeChildren::Done(new_children);
                    tree_state.open(id.clone());
                    true
                } else {
                    false
                }
            }
        }
    }
}
