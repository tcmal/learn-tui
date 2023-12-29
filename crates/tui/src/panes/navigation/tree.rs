use tui_tree_widget::TreeItem;

use crate::store::{ContentIdx, CourseIdx, Store};

/// Our navigation tree, but with only IDs, loading information, etc.
/// This is a sort of 'abstract' tree that gets compiled into a [`TreeItem`] which is then rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavTree {
    /// An item which may have children at some point
    Node {
        ty: NodeTy,
        children: NavTreeChildren,
    },
    /// An item which will never have children
    ContentLeaf { content_idx: ContentIdx },

    /// A placeholder to show that the whole tree is loading.
    Loading,
}

/// The type of a node - either course or content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeTy {
    Course(CourseIdx),
    Content(ContentIdx),
}

/// The state of the children of a node
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavTreeChildren {
    /// Children have been loaded
    Done(Vec<NavTree>),

    /// Loading requested but not finished
    Loading,

    /// This node can have children, but they have not been requested yet
    NotRequested,
}

/// Identifies a specific item in the tree. Used for selection, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreeId {
    Course(CourseIdx),
    CourseLoading(CourseIdx),
    Content(CourseIdx),
    ContentLoading(CourseIdx),
    Loading,
}

impl NavTree {
    /// Get the corresponding NavTree element for some selector, used by [`tui_tree_widget`].
    pub fn navigate_mut<'a>(leafs: &'a mut [Self], ids: &[TreeId]) -> &'a mut NavTree {
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
                NavTree::Node {
                    children: NavTreeChildren::Done(cs),
                    ..
                } => Self::navigate_mut(cs, remaining_search),
                _ => unreachable!(),
            }
        }
    }

    fn matches(&self, id: TreeId) -> bool {
        match (self, id) {
            (NavTree::Node { ty, .. }, id) => ty.matches(id),
            (NavTree::ContentLeaf { content_idx }, TreeId::Content(idx))
            | (NavTree::ContentLeaf { content_idx }, TreeId::ContentLoading(idx)) => {
                *content_idx == idx
            }
            _ => false,
        }
    }

    pub fn as_treeitem(&self, store: &Store) -> TreeItem<'static, TreeId> {
        const LOADING: &str = "Loading...";
        match self {
            // base case: nodes with no children
            NavTree::ContentLeaf { content_idx } => TreeItem::new_leaf(
                TreeId::Content(*content_idx),
                store.content(*content_idx).title.to_string(),
            ),
            NavTree::Loading => TreeItem::new_leaf(TreeId::Loading, LOADING),
            NavTree::Node {
                ty,
                children: NavTreeChildren::NotRequested,
            } => ty.treeitem_leaf(store),

            // loading text
            NavTree::Node {
                ty,
                children: NavTreeChildren::Loading,
            } => ty.treeitem_with(store, vec![TreeItem::new_leaf(ty.loading_id(), LOADING)]),

            // nodes with children
            NavTree::Node {
                ty,
                children: NavTreeChildren::Done(children),
            } => ty.treeitem_with(
                store,
                children.iter().map(|nt| nt.as_treeitem(store)).collect(),
            ),
        }
    }

    pub fn id(&self) -> TreeId {
        match self {
            NavTree::Node { ty, .. } => ty.id(),
            NavTree::ContentLeaf { content_idx } => TreeId::Content(*content_idx),
            NavTree::Loading => TreeId::Loading,
        }
    }
}

impl NodeTy {
    /// Send a request for this node's children
    pub fn request_children(&self, store: &Store) {
        match self {
            NodeTy::Course(i) => store.request_course_content(*i),
            NodeTy::Content(i) => store.request_content_children(*i),
        }
    }

    /// Check if the children have been loaded, and if so return them
    pub fn new_children_loaded(&self, store: &Store) -> Option<Vec<NavTree>> {
        let idxs = match self {
            NodeTy::Course(i) => store.course_content(*i),
            NodeTy::Content(i) => store.content_children(*i),
        }?;
        Some(
            idxs.map(|content_idx| {
                let content = store.content(content_idx);

                if content.is_container() {
                    NavTree::Node {
                        ty: NodeTy::Content(content_idx),
                        children: NavTreeChildren::NotRequested,
                    }
                } else {
                    NavTree::ContentLeaf { content_idx }
                }
            })
            .collect(),
        )
    }

    /// Check if this node matches the given ID
    fn matches(&self, id: TreeId) -> bool {
        match (self, id) {
            (NodeTy::Course(i), TreeId::Course(j))
            | (NodeTy::Course(i), TreeId::CourseLoading(j))
            | (NodeTy::Content(i), TreeId::Content(j))
            | (NodeTy::Content(i), TreeId::ContentLoading(j)) => *i == j,
            _ => false,
        }
    }

    /// Get the display name for this node.
    fn display_name(&self, store: &Store) -> String {
        match self {
            NodeTy::Course(i) => store.course(*i).name.clone(),
            NodeTy::Content(i) => store.content(*i).title.clone(),
        }
    }

    /// Create a treeitem for this node with the given children.
    fn treeitem_with(
        &self,
        store: &Store,
        children: Vec<TreeItem<'static, TreeId>>,
    ) -> TreeItem<'static, TreeId> {
        TreeItem::new(self.id(), self.display_name(store), children).unwrap()
    }

    /// Create a leaf treeitem for this node
    fn treeitem_leaf(&self, store: &Store) -> TreeItem<'static, TreeId> {
        TreeItem::new_leaf(self.id(), self.display_name(store))
    }

    /// Get the ID for this node
    fn id(&self) -> TreeId {
        match self {
            NodeTy::Course(i) => TreeId::Course(*i),
            NodeTy::Content(i) => TreeId::Content(*i),
        }
    }

    /// Get the ID for a loading element beneath this node.
    fn loading_id(&self) -> TreeId {
        match self {
            NodeTy::Course(i) => TreeId::CourseLoading(*i),
            NodeTy::Content(i) => TreeId::ContentLoading(*i),
        }
    }
}

impl Default for TreeId {
    fn default() -> Self {
        Self::Loading
    }
}
