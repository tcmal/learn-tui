use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::Rect, Frame};
use tui_tree_widget::{Tree, TreeItem, TreeState};

use super::{Action, Document, Page};
use crate::store::{ContentIdx, CourseIdx, Store};

impl Default for TreeId {
    fn default() -> Self {
        Self::Loading
    }
}

#[derive(Default)]
pub struct NavigationPage {
    tree_state: TreeState<TreeId>,
    nav_tree: Vec<NavTree>,
    cached_view_tree: Option<Vec<TreeItem<'static, TreeId>>>,
}

impl Page for NavigationPage {
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

    fn handle_key(&mut self, store: &Store, key: KeyEvent) -> Result<Action> {
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
                    NavTree::CourseNode {
                        children: NavTreeChildren::Done(_),
                        ..
                    } => self.tree_state.toggle(sel),
                    NavTree::ContentNode {
                        children: NavTreeChildren::Done(_),
                        ..
                    } => self.tree_state.toggle(sel),

                    // request loading
                    NavTree::CourseNode {
                        course_idx,
                        children: children @ NavTreeChildren::NotRequested,
                    } => {
                        store.request_course_content(*course_idx);
                        *children = NavTreeChildren::Loading;
                        self.cached_view_tree = None;
                    }
                    NavTree::ContentNode {
                        content_idx,
                        children: children @ NavTreeChildren::NotRequested,
                    } => {
                        store.request_content_children(*content_idx);
                        *children = NavTreeChildren::Loading;
                        self.cached_view_tree = None;
                    }

                    NavTree::ContentLeaf { content_idx } => {
                        return Ok(Action::Show(Document::Content(*content_idx)));
                    }

                    // do nothing on loading stuff
                    NavTree::CourseNode {
                        children: NavTreeChildren::Loading,
                        ..
                    } => (),
                    NavTree::ContentNode {
                        children: NavTreeChildren::Loading,
                        ..
                    } => (),
                    NavTree::Loading => (),
                }
            }
            _ => (),
        };

        Ok(Action::None)
    }
}

impl NavigationPage {
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
            if let Some(courses) = store.my_courses() {
                // done loading
                self.nav_tree = (0..courses.len())
                    .map(|course_idx| NavTree::CourseNode {
                        course_idx,
                        children: NavTreeChildren::NotRequested,
                    })
                    .collect();
                self.tree_state.select(vec![TreeId::Course(0)]);
                changed = true;
            } else {
                // still loading
                return false;
            }
        }

        // loaded/partially loaded tree
        for item in self.nav_tree.iter_mut() {
            let NavTree::CourseNode { course_idx, .. } = &item else {
                panic!("top level in tree is not course node");
            };
            changed |= Self::refresh_subtree(
                &mut self.tree_state,
                store,
                &mut vec![TreeId::Course(*course_idx)],
                item,
            );
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
            NavTree::CourseNode {
                children: NavTreeChildren::NotRequested,
                ..
            }
            | NavTree::ContentNode {
                children: NavTreeChildren::NotRequested,
                ..
            } => false,
            NavTree::ContentLeaf { .. } => false,
            NavTree::Loading => false,

            // recursively refresh loaded subtrees
            NavTree::CourseNode {
                children: NavTreeChildren::Done(cs),
                ..
            }
            | NavTree::ContentNode {
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
                .fold(false, |acc, changed| acc | changed),

            // check if loading subtrees have finished
            NavTree::CourseNode {
                course_idx,
                children: children @ NavTreeChildren::Loading,
            } => {
                if let Some(new_children) = store.course_content(*course_idx) {
                    *children = NavTreeChildren::Done(
                        new_children
                            .map(|content_idx| {
                                let content = store.content(content_idx);

                                if content.is_container() {
                                    NavTree::ContentNode {
                                        content_idx,
                                        children: NavTreeChildren::NotRequested,
                                    }
                                } else {
                                    NavTree::ContentLeaf { content_idx }
                                }
                            })
                            .collect(),
                    );
                    tree_state.open(id.clone());
                    true
                } else {
                    false
                }
            }
            NavTree::ContentNode {
                content_idx,
                children: children @ NavTreeChildren::Loading,
            } => {
                if let Some(new_children) = store.content_children(*content_idx) {
                    *children = NavTreeChildren::Done(
                        new_children
                            .map(|content_idx| {
                                let content = store.content(content_idx);

                                if content.is_container() {
                                    NavTree::ContentNode {
                                        content_idx,
                                        children: NavTreeChildren::NotRequested,
                                    }
                                } else {
                                    NavTree::ContentLeaf { content_idx }
                                }
                            })
                            .collect(),
                    );
                    tree_state.open(id.clone());
                    true
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavTree {
    CourseNode {
        course_idx: CourseIdx,
        children: NavTreeChildren,
    },
    ContentNode {
        content_idx: ContentIdx,
        children: NavTreeChildren,
    },
    ContentLeaf {
        content_idx: ContentIdx,
    },
    Loading,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavTreeChildren {
    Done(Vec<NavTree>),
    Loading,
    NotRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreeId {
    Course(CourseIdx),
    CourseLoading(CourseIdx),
    Content(CourseIdx),
    ContentLoading(CourseIdx),
    Loading,
}

impl NavTree {
    fn navigate_mut<'a>(leafs: &'a mut [Self], ids: &[TreeId]) -> &'a mut NavTree {
        if ids.is_empty() {
            panic!("attempt to get navtree with invalid id");
        }
        let next = leafs
            .iter_mut()
            .find(|x| x.matches(ids[0]))
            .expect("invalid id for navtree");
        if ids.len() == 1 || matches!(ids[1], TreeId::CourseLoading(_) | TreeId::ContentLoading(_))
        {
            next
        } else {
            let remaining_search = &ids[1..];
            match next {
                NavTree::CourseNode {
                    children: NavTreeChildren::Done(cs),
                    ..
                } => Self::navigate_mut(cs, remaining_search),
                NavTree::ContentNode {
                    children: NavTreeChildren::Done(cs),
                    ..
                } => Self::navigate_mut(cs, remaining_search),
                _ => unreachable!(),
            }
        }
    }

    fn matches(&self, id: TreeId) -> bool {
        match (self, id) {
            (NavTree::CourseNode { course_idx, .. }, TreeId::Course(idx)) => *course_idx == idx,
            (NavTree::CourseNode { course_idx, .. }, TreeId::CourseLoading(idx)) => {
                *course_idx == idx
            }
            (NavTree::ContentNode { content_idx, .. }, TreeId::Content(idx)) => *content_idx == idx,
            (NavTree::ContentNode { content_idx, .. }, TreeId::ContentLoading(idx)) => {
                *content_idx == idx
            }
            (NavTree::ContentLeaf { content_idx }, TreeId::Content(idx)) => *content_idx == idx,
            (NavTree::ContentLeaf { content_idx }, TreeId::ContentLoading(idx)) => {
                *content_idx == idx
            }
            _ => false,
        }
    }

    fn as_treeitem(&self, store: &Store) -> TreeItem<'static, TreeId> {
        const LOADING: &str = "Loading...";
        match self {
            // base case: nodes with no children
            NavTree::ContentLeaf { content_idx } => TreeItem::new_leaf(
                TreeId::Content(*content_idx),
                store.content(*content_idx).title.to_string(),
            ),
            NavTree::Loading => TreeItem::new_leaf(TreeId::Loading, LOADING),
            NavTree::CourseNode {
                course_idx,
                children: NavTreeChildren::NotRequested,
            } => TreeItem::new_leaf(
                TreeId::Course(*course_idx),
                store.course(*course_idx).name.to_string(),
            ),
            NavTree::ContentNode {
                content_idx,
                children: NavTreeChildren::NotRequested,
            } => TreeItem::new_leaf(
                TreeId::Content(*content_idx),
                store.content(*content_idx).title.to_string(),
            ),

            // loading text
            NavTree::CourseNode {
                course_idx,
                children: NavTreeChildren::Loading,
            } => TreeItem::new(
                TreeId::Course(*course_idx),
                store.course(*course_idx).name.to_string(),
                vec![TreeItem::new_leaf(
                    TreeId::CourseLoading(*course_idx),
                    LOADING,
                )],
            )
            .unwrap(),
            NavTree::ContentNode {
                content_idx,
                children: NavTreeChildren::Loading,
            } => TreeItem::new(
                TreeId::Content(*content_idx),
                store.content(*content_idx).title.to_string(),
                vec![TreeItem::new_leaf(
                    TreeId::ContentLoading(*content_idx),
                    LOADING,
                )],
            )
            .unwrap(),

            // nodes with children
            NavTree::CourseNode {
                course_idx,
                children: NavTreeChildren::Done(children),
            } => TreeItem::new(
                TreeId::Course(*course_idx),
                store.course(*course_idx).name.to_string(),
                children.iter().map(|nt| nt.as_treeitem(store)).collect(),
            )
            .unwrap(),
            NavTree::ContentNode {
                content_idx,
                children: NavTreeChildren::Done(children),
            } => TreeItem::new(
                TreeId::Content(*content_idx),
                store.content(*content_idx).title.to_string(),
                children.iter().map(|nt| nt.as_treeitem(store)).collect(),
            )
            .unwrap(),
        }
    }

    fn id(&self) -> TreeId {
        match self {
            NavTree::CourseNode { course_idx, .. } => TreeId::Course(*course_idx),
            NavTree::ContentNode { content_idx, .. } => TreeId::Content(*content_idx),
            NavTree::ContentLeaf { content_idx } => TreeId::Content(*content_idx),
            NavTree::Loading => TreeId::Loading,
        }
    }
}
