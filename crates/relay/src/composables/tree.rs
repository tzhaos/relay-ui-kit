use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    hash::Hash,
    rc::Rc,
};

use gpui::{App, Context};

use crate::{Effect, Memo, ReactiveContextExt, Signal, batch};

/// A visible tree node projected from the latest source tree.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectedTreeNode<T, K> {
    key: K,
    item: T,
    depth: usize,
    parent: Option<K>,
    has_children: bool,
    is_expanded: bool,
}

impl<T, K> ProjectedTreeNode<T, K> {
    /// Return this node's stable key.
    pub fn key(&self) -> &K {
        &self.key
    }

    /// Return the projected item for this visible node.
    pub fn item(&self) -> &T {
        &self.item
    }

    /// Return this node's current visible depth.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Return this node's parent key, if any.
    pub fn parent(&self) -> Option<&K> {
        self.parent.as_ref()
    }

    /// Return whether this node currently has any children.
    pub fn has_children(&self) -> bool {
        self.has_children
    }

    /// Return whether this node is currently expanded.
    pub fn is_expanded(&self) -> bool {
        self.is_expanded
    }
}

/// A source-driven visible tree projection with managed expansion state.
///
/// `TreeProjection` keeps the latest visible nodes, visible key order, and
/// expanded key state in sync with a tree source. This is the migration
/// composable for tree- and panel-like GPUI surfaces that need:
///
/// - a flattened visible node list;
/// - stable visible-key order for selection/navigation models;
/// - explicit expand/collapse/toggle helpers;
/// - ancestor reveal without host-side tree traversal glue.
pub struct TreeProjection<T, K> {
    visible_nodes: Signal<Vec<ProjectedTreeNode<T, K>>>,
    visible_keys: Signal<Vec<K>>,
    all_keys: Signal<Vec<K>>,
    expanded_keys: Signal<Vec<K>>,
    expandable_keys: Signal<Vec<K>>,
    parent_by_key: Signal<HashMap<K, Option<K>>>,
    has_visible_nodes: Memo<bool>,
    visible_count: Memo<usize>,
    keyed: Rc<RefCell<HashMap<K, Signal<bool>>>>,
    _effect: Rc<Effect>,
}

/// Create a source-driven tree projection.
///
/// `roots` declares the latest tree source for this entity. `key` must return
/// a stable unique key for every node in the tree. `children` returns the
/// current child nodes for a parent node.
pub fn use_tree_projection<T, K, Owner, RootsFn, KeyFn, ChildrenFn, Children>(
    cx: &mut Context<Owner>,
    expanded: impl IntoIterator<Item = K>,
    roots: RootsFn,
    key: KeyFn,
    children: ChildrenFn,
) -> TreeProjection<T, K>
where
    Owner: 'static,
    T: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + PartialEq + 'static,
    RootsFn: Fn(&App) -> Vec<T> + 'static,
    KeyFn: Fn(&T) -> K + 'static,
    ChildrenFn: Fn(&T) -> Children + 'static,
    Children: IntoIterator<Item = T>,
{
    TreeProjection::new(cx, expanded, roots, key, children)
}

impl<T, K> TreeProjection<T, K>
where
    T: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    /// Create a source-driven tree projection.
    pub fn new<Owner, RootsFn, KeyFn, ChildrenFn, Children>(
        cx: &mut Context<Owner>,
        expanded: impl IntoIterator<Item = K>,
        roots: RootsFn,
        key: KeyFn,
        children: ChildrenFn,
    ) -> Self
    where
        Owner: 'static,
        RootsFn: Fn(&App) -> Vec<T> + 'static,
        KeyFn: Fn(&T) -> K + 'static,
        ChildrenFn: Fn(&T) -> Children + 'static,
        Children: IntoIterator<Item = T>,
    {
        let visible_nodes = Signal::new(cx, Vec::new());
        let visible_keys = Signal::new(cx, Vec::new());
        let all_keys = Signal::new(cx, Vec::new());
        let expanded_keys = Signal::new(cx, dedupe_keys(expanded));
        let expandable_keys = Signal::new(cx, Vec::new());
        let parent_by_key = Signal::new(cx, HashMap::new());
        let keyed: Rc<RefCell<HashMap<K, Signal<bool>>>> = Rc::default();

        let visible_nodes_for_has = visible_nodes.clone();
        let visible_nodes_for_count = visible_nodes.clone();
        let latest_projection: Rc<RefCell<Option<ProjectionSnapshot<T, K>>>> =
            Rc::new(RefCell::new(None));
        let latest_projection_for_sources = latest_projection.clone();
        let expanded_keys_for_sources = expanded_keys.clone();
        let visible_nodes_for_effect = visible_nodes.clone();
        let visible_keys_for_effect = visible_keys.clone();
        let all_keys_for_effect = all_keys.clone();
        let expandable_keys_for_effect = expandable_keys.clone();
        let parent_by_key_for_effect = parent_by_key.clone();
        let expanded_keys_for_effect = expanded_keys.clone();
        let keyed_for_effect = keyed.clone();

        let effect = cx.watch(
            move |cx| {
                let roots = roots(cx);
                let expanded = expanded_keys_for_sources.get(cx);
                let expanded = expanded.into_iter().collect::<HashSet<_>>();
                *latest_projection_for_sources.borrow_mut() = Some(build_projection(
                    roots.as_slice(),
                    &expanded,
                    &key,
                    &children,
                ));
            },
            move |cx| {
                let Some(snapshot) = latest_projection.borrow_mut().take() else {
                    return;
                };
                let normalized_expanded = normalize_keys_to_order(
                    expanded_keys_for_effect.get_untracked(),
                    snapshot.expandable_keys.as_slice(),
                );

                batch(cx, |cx| {
                    visible_nodes_for_effect.set(cx, snapshot.visible_nodes.clone());
                    visible_keys_for_effect.set(cx, snapshot.visible_keys.clone());
                    all_keys_for_effect.set(cx, snapshot.all_keys.clone());
                    expandable_keys_for_effect.set(cx, snapshot.expandable_keys.clone());
                    parent_by_key_for_effect.set(cx, snapshot.parent_by_key.clone());
                    apply_keyed_state(
                        &keyed_for_effect,
                        cx,
                        expanded_keys_for_effect.get_untracked().as_slice(),
                        normalized_expanded.as_slice(),
                    );
                    retain_keyed_signals(
                        &keyed_for_effect,
                        snapshot.expandable_keys.iter().cloned(),
                    );
                    expanded_keys_for_effect.set(cx, normalized_expanded);
                });
            },
        );

        Self {
            visible_nodes,
            visible_keys,
            all_keys,
            expanded_keys,
            expandable_keys,
            parent_by_key,
            has_visible_nodes: Memo::new(cx, move |cx| !visible_nodes_for_has.get(cx).is_empty()),
            visible_count: Memo::new(cx, move |cx| {
                visible_nodes_for_count.read(cx, |visible_nodes| visible_nodes.len())
            }),
            keyed,
            _effect: Rc::new(effect),
        }
    }

    /// Return the signal holding the latest visible nodes.
    pub fn visible_nodes_signal(&self) -> &Signal<Vec<ProjectedTreeNode<T, K>>> {
        &self.visible_nodes
    }

    /// Read the latest visible nodes with dependency tracking.
    pub fn read_visible_nodes<R>(
        &self,
        cx: &App,
        f: impl FnOnce(&[ProjectedTreeNode<T, K>]) -> R,
    ) -> R {
        self.visible_nodes
            .read(cx, |visible_nodes| f(visible_nodes.as_slice()))
    }

    /// Clone the latest visible nodes with dependency tracking.
    pub fn visible_nodes(&self, cx: &App) -> Vec<ProjectedTreeNode<T, K>> {
        self.visible_nodes.get(cx)
    }

    /// Clone the latest visible nodes without dependency tracking.
    pub fn visible_nodes_untracked(&self) -> Vec<ProjectedTreeNode<T, K>> {
        self.visible_nodes.get_untracked()
    }

    /// Return the signal holding the latest visible key order.
    pub fn visible_keys_signal(&self) -> &Signal<Vec<K>> {
        &self.visible_keys
    }

    /// Clone the latest visible keys with dependency tracking.
    pub fn visible_keys(&self, cx: &App) -> Vec<K> {
        self.visible_keys.get(cx)
    }

    /// Clone the latest visible keys without dependency tracking.
    pub fn visible_keys_untracked(&self) -> Vec<K> {
        self.visible_keys.get_untracked()
    }

    /// Return the signal holding every key in the latest tree snapshot.
    pub fn all_keys_signal(&self) -> &Signal<Vec<K>> {
        &self.all_keys
    }

    /// Clone every key in the latest tree snapshot with dependency tracking.
    pub fn all_keys(&self, cx: &App) -> Vec<K> {
        self.all_keys.get(cx)
    }

    /// Clone every key in the latest tree snapshot without dependency tracking.
    pub fn all_keys_untracked(&self) -> Vec<K> {
        self.all_keys.get_untracked()
    }

    /// Return the signal holding the latest expanded keys.
    pub fn expanded_keys_signal(&self) -> &Signal<Vec<K>> {
        &self.expanded_keys
    }

    /// Clone the latest expanded keys with dependency tracking.
    pub fn expanded_keys(&self, cx: &App) -> Vec<K> {
        self.expanded_keys.get(cx)
    }

    /// Clone the latest expanded keys without dependency tracking.
    pub fn expanded_keys_untracked(&self) -> Vec<K> {
        self.expanded_keys.get_untracked()
    }

    /// Return a memo that is `true` when any visible nodes exist.
    pub fn has_visible_nodes(&self) -> &Memo<bool> {
        &self.has_visible_nodes
    }

    /// Return a memo with the latest visible node count.
    pub fn visible_count(&self) -> &Memo<usize> {
        &self.visible_count
    }

    /// Return whether the given key is currently expandable.
    pub fn is_expandable(&self, cx: &App, key: &K) -> bool {
        self.expandable_keys.read(cx, |expandable| {
            expandable
                .iter()
                .any(|expandable_key| expandable_key == key)
        })
    }

    /// Return whether the given key is currently expanded.
    pub fn is_expanded(&self, cx: &mut App, key: K) -> bool {
        self.key_signal(cx, key).get(cx)
    }

    /// Expand a single key.
    pub fn expand(&self, cx: &mut App, key: K) -> bool {
        if !self.is_expandable_untracked(&key) {
            return false;
        }

        let mut expanded = self.expanded_keys.get_untracked();
        expanded.push(key);
        self.set_expanded_keys(cx, expanded)
    }

    /// Collapse a single key.
    pub fn collapse(&self, cx: &mut App, key: K) -> bool {
        if !self
            .expanded_keys
            .peek(|expanded| expanded.iter().any(|expanded_key| expanded_key == &key))
        {
            return false;
        }

        let expanded = self
            .expanded_keys
            .get_untracked()
            .into_iter()
            .filter(|expanded_key| expanded_key != &key)
            .collect::<Vec<_>>();
        self.set_expanded_keys(cx, expanded)
    }

    /// Toggle a single key between expanded and collapsed.
    pub fn toggle(&self, cx: &mut App, key: K) -> bool {
        if self
            .expanded_keys
            .peek(|expanded| expanded.iter().any(|expanded_key| expanded_key == &key))
        {
            self.collapse(cx, key)
        } else {
            self.expand(cx, key)
        }
    }

    /// Expand every currently expandable key in tree order.
    pub fn expand_all(&self, cx: &mut App) -> bool {
        self.set_expanded_keys(cx, self.expandable_keys.get_untracked())
    }

    /// Collapse every expanded key.
    pub fn collapse_all(&self, cx: &mut App) -> bool {
        self.set_expanded_keys(cx, Vec::new())
    }

    /// Expand the ancestor chain for `key` so it becomes visible.
    pub fn reveal(&self, cx: &mut App, key: K) -> bool {
        let Some(mut parent) = self.parent_key_untracked(&key) else {
            return false;
        };
        let mut expanded = self.expanded_keys.get_untracked();
        let mut changed = false;

        while let Some(current) = parent {
            if self.is_expandable_untracked(&current)
                && !expanded.iter().any(|expanded_key| expanded_key == &current)
            {
                expanded.push(current.clone());
                changed = true;
            }
            parent = self.parent_key_untracked(&current).flatten();
        }

        if !changed {
            return false;
        }

        self.set_expanded_keys(cx, expanded)
    }

    /// Reconcile the current expanded keys against the latest tree snapshot.
    pub fn reconcile_now(&self, cx: &mut App) -> bool {
        let normalized = normalize_keys_to_order(
            self.expanded_keys.get_untracked(),
            self.expandable_keys.get_untracked().as_slice(),
        );
        self.set_expanded_keys(cx, normalized)
    }

    fn set_expanded_keys(&self, cx: &mut App, expanded: Vec<K>) -> bool {
        let expanded =
            normalize_keys_to_order(expanded, self.expandable_keys.get_untracked().as_slice());
        let previous = self.expanded_keys.get_untracked();
        if previous == expanded {
            return false;
        }

        batch(cx, |cx| {
            apply_keyed_state(&self.keyed, cx, previous.as_slice(), expanded.as_slice());
            self.expanded_keys.set(cx, expanded);
        });

        true
    }

    fn is_expandable_untracked(&self, key: &K) -> bool {
        self.expandable_keys.peek(|expandable| {
            expandable
                .iter()
                .any(|expandable_key| expandable_key == key)
        })
    }

    fn parent_key_untracked(&self, key: &K) -> Option<Option<K>> {
        self.parent_by_key.peek(|parents| parents.get(key).cloned())
    }

    fn key_signal(&self, cx: &mut App, key: K) -> Signal<bool> {
        if let Some(signal) = self.keyed.borrow().get(&key) {
            return signal.clone();
        }

        let is_expanded = self
            .expanded_keys
            .peek(|expanded| expanded.iter().any(|expanded_key| expanded_key == &key));
        let signal = Signal::new(cx, is_expanded);
        self.keyed.borrow_mut().insert(key, signal.clone());
        signal
    }
}

impl<T, K> Clone for TreeProjection<T, K> {
    fn clone(&self) -> Self {
        Self {
            visible_nodes: self.visible_nodes.clone(),
            visible_keys: self.visible_keys.clone(),
            all_keys: self.all_keys.clone(),
            expanded_keys: self.expanded_keys.clone(),
            expandable_keys: self.expandable_keys.clone(),
            parent_by_key: self.parent_by_key.clone(),
            has_visible_nodes: self.has_visible_nodes.clone(),
            visible_count: self.visible_count.clone(),
            keyed: self.keyed.clone(),
            _effect: self._effect.clone(),
        }
    }
}

struct ProjectionSnapshot<T, K> {
    visible_nodes: Vec<ProjectedTreeNode<T, K>>,
    visible_keys: Vec<K>,
    all_keys: Vec<K>,
    expandable_keys: Vec<K>,
    parent_by_key: HashMap<K, Option<K>>,
}

fn build_projection<T, K, KeyFn, ChildrenFn, Children>(
    roots: &[T],
    expanded: &HashSet<K>,
    key: &KeyFn,
    children: &ChildrenFn,
) -> ProjectionSnapshot<T, K>
where
    T: Clone,
    K: Clone + Eq + Hash,
    KeyFn: Fn(&T) -> K,
    ChildrenFn: Fn(&T) -> Children,
    Children: IntoIterator<Item = T>,
{
    let mut snapshot = ProjectionSnapshot {
        visible_nodes: Vec::new(),
        visible_keys: Vec::new(),
        all_keys: Vec::new(),
        expandable_keys: Vec::new(),
        parent_by_key: HashMap::new(),
    };

    for root in roots {
        visit_tree_node(root, None, 0, true, expanded, key, children, &mut snapshot);
    }

    snapshot
}

fn visit_tree_node<T, K, KeyFn, ChildrenFn, Children>(
    node: &T,
    parent: Option<K>,
    depth: usize,
    is_visible: bool,
    expanded: &HashSet<K>,
    key: &KeyFn,
    children: &ChildrenFn,
    snapshot: &mut ProjectionSnapshot<T, K>,
) where
    T: Clone,
    K: Clone + Eq + Hash,
    KeyFn: Fn(&T) -> K,
    ChildrenFn: Fn(&T) -> Children,
    Children: IntoIterator<Item = T>,
{
    let node_key = key(node);
    let child_nodes = children(node).into_iter().collect::<Vec<_>>();
    let has_children = !child_nodes.is_empty();
    let is_expanded = has_children && expanded.contains(&node_key);

    snapshot.all_keys.push(node_key.clone());
    snapshot
        .parent_by_key
        .insert(node_key.clone(), parent.clone());
    if has_children {
        snapshot.expandable_keys.push(node_key.clone());
    }
    if is_visible {
        snapshot.visible_keys.push(node_key.clone());
        snapshot.visible_nodes.push(ProjectedTreeNode {
            key: node_key.clone(),
            item: node.clone(),
            depth,
            parent: parent.clone(),
            has_children,
            is_expanded,
        });
    }

    for child in &child_nodes {
        visit_tree_node(
            child,
            Some(node_key.clone()),
            depth + 1,
            is_visible && is_expanded,
            expanded,
            key,
            children,
            snapshot,
        );
    }
}

fn apply_keyed_state<K>(
    keyed: &Rc<RefCell<HashMap<K, Signal<bool>>>>,
    cx: &mut App,
    previous: &[K],
    next: &[K],
) where
    K: Clone + Eq + Hash,
{
    let previous = previous.iter().cloned().collect::<HashSet<_>>();
    let next = next.iter().cloned().collect::<HashSet<_>>();
    let keyed = keyed.borrow();

    for removed in previous.difference(&next) {
        if let Some(signal) = keyed.get(removed) {
            signal.set(cx, false);
        }
    }

    for added in next.difference(&previous) {
        if let Some(signal) = keyed.get(added) {
            signal.set(cx, true);
        }
    }
}

fn retain_keyed_signals<K>(
    keyed: &Rc<RefCell<HashMap<K, Signal<bool>>>>,
    keys: impl IntoIterator<Item = K>,
) where
    K: Clone + Eq + Hash,
{
    let keys = keys.into_iter().collect::<HashSet<_>>();
    keyed.borrow_mut().retain(|key, _| keys.contains(key));
}

fn dedupe_keys<K>(keys: impl IntoIterator<Item = K>) -> Vec<K>
where
    K: Clone + Eq + Hash,
{
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();

    for key in keys {
        if seen.insert(key.clone()) {
            deduped.push(key);
        }
    }

    deduped
}

fn normalize_keys_to_order<K>(keys: impl IntoIterator<Item = K>, ordered_keys: &[K]) -> Vec<K>
where
    K: Clone + Eq + Hash,
{
    let input = dedupe_keys(keys);
    if input.is_empty() {
        return input;
    }
    if ordered_keys.is_empty() {
        return Vec::new();
    }

    let input_set = input.iter().cloned().collect::<HashSet<_>>();
    ordered_keys
        .iter()
        .filter(|key| input_set.contains(*key))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use gpui::{AppContext, Context, TestAppContext};

    use crate::init;

    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TreeNode {
        id: u64,
        label: &'static str,
        children: Vec<TreeNode>,
    }

    impl TreeNode {
        fn file(id: u64, label: &'static str) -> Self {
            Self {
                id,
                label,
                children: Vec::new(),
            }
        }

        fn dir(id: u64, label: &'static str, children: Vec<TreeNode>) -> Self {
            Self {
                id,
                label,
                children,
            }
        }
    }

    #[test]
    fn projected_tree_node_exposes_visible_shape() {
        let node = ProjectedTreeNode {
            key: 1_u64,
            item: TreeNode::file(1, "main.rs"),
            depth: 2,
            parent: Some(9),
            has_children: false,
            is_expanded: false,
        };

        assert_eq!(node.key(), &1);
        assert_eq!(node.item().label, "main.rs");
        assert_eq!(node.depth(), 2);
        assert_eq!(node.parent(), Some(&9));
        assert!(!node.has_children());
        assert!(!node.is_expanded());
    }

    #[gpui::test]
    fn tree_projection_flattens_visible_nodes_from_expansion_state(cx: &mut TestAppContext) {
        struct Host {
            tree: Signal<Vec<TreeNode>>,
            projection: TreeProjection<TreeNode, u64>,
        }

        impl Host {
            fn new(cx: &mut Context<Self>) -> Self {
                init(cx);
                let tree = Signal::new(
                    cx,
                    vec![
                        TreeNode::dir(
                            1,
                            "src",
                            vec![TreeNode::file(2, "main.rs"), TreeNode::file(3, "lib.rs")],
                        ),
                        TreeNode::dir(4, "docs", vec![TreeNode::file(5, "guide.md")]),
                    ],
                );
                let tree_for_projection = tree.clone();
                let projection = use_tree_projection(
                    cx,
                    Vec::<u64>::new(),
                    move |cx| tree_for_projection.get(cx),
                    |node| node.id,
                    |node| node.children.clone(),
                );

                Self { tree, projection }
            }
        }

        let entity = cx.update(|cx| cx.new(Host::new));

        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.tree.get(cx).len(), 2);
            assert_eq!(host.projection.visible_keys(cx), vec![1, 4]);
            assert_eq!(
                host.projection
                    .visible_nodes(cx)
                    .into_iter()
                    .map(|node| (
                        node.item.label,
                        node.depth,
                        node.has_children,
                        node.is_expanded
                    ))
                    .collect::<Vec<_>>(),
                vec![("src", 0, true, false), ("docs", 0, true, false)]
            );
        });

        cx.update_entity(&entity, |host, cx| {
            assert!(host.projection.expand(cx, 1));
        });

        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.projection.visible_keys(cx), vec![1, 2, 3, 4]);
            assert_eq!(
                host.projection
                    .visible_nodes(cx)
                    .into_iter()
                    .map(|node| (node.item.label, node.depth))
                    .collect::<Vec<_>>(),
                vec![("src", 0), ("main.rs", 1), ("lib.rs", 1), ("docs", 0)]
            );
            assert_eq!(host.projection.expanded_keys(cx), vec![1]);
        });
    }

    #[gpui::test]
    fn tree_projection_reveal_expands_ancestor_chain(cx: &mut TestAppContext) {
        struct Host {
            projection: TreeProjection<TreeNode, u64>,
        }

        impl Host {
            fn new(cx: &mut Context<Self>) -> Self {
                init(cx);
                let tree = Signal::new(
                    cx,
                    vec![TreeNode::dir(
                        1,
                        "root",
                        vec![TreeNode::dir(2, "src", vec![TreeNode::file(3, "main.rs")])],
                    )],
                );
                let tree_for_projection = tree.clone();
                let projection = use_tree_projection(
                    cx,
                    Vec::<u64>::new(),
                    move |cx| tree_for_projection.get(cx),
                    |node| node.id,
                    |node| node.children.clone(),
                );

                Self { projection }
            }
        }

        let entity = cx.update(|cx| cx.new(Host::new));

        cx.update_entity(&entity, |host, cx| {
            assert!(host.projection.reveal(cx, 3));
        });

        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.projection.expanded_keys(cx), vec![1, 2]);
            assert_eq!(host.projection.visible_keys(cx), vec![1, 2, 3]);
            assert_eq!(
                host.projection
                    .visible_nodes(cx)
                    .into_iter()
                    .map(|node| node.item.label)
                    .collect::<Vec<_>>(),
                vec!["root", "src", "main.rs"]
            );
        });
    }

    #[gpui::test]
    fn tree_projection_reconciles_expanded_keys_when_source_changes(cx: &mut TestAppContext) {
        struct Host {
            tree: Signal<Vec<TreeNode>>,
            projection: TreeProjection<TreeNode, u64>,
        }

        impl Host {
            fn new(cx: &mut Context<Self>) -> Self {
                init(cx);
                let tree = Signal::new(
                    cx,
                    vec![TreeNode::dir(1, "src", vec![TreeNode::file(2, "main.rs")])],
                );
                let tree_for_projection = tree.clone();
                let projection = use_tree_projection(
                    cx,
                    [1],
                    move |cx| tree_for_projection.get(cx),
                    |node| node.id,
                    |node| node.children.clone(),
                );

                Self { tree, projection }
            }
        }

        let entity = cx.update(|cx| cx.new(Host::new));

        cx.update_entity(&entity, |host, cx| {
            host.tree.set(cx, vec![TreeNode::dir(9, "docs", vec![])]);
        });

        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.projection.all_keys(cx), vec![9]);
            assert_eq!(host.projection.expanded_keys(cx), Vec::<u64>::new());
            assert_eq!(host.projection.visible_keys(cx), vec![9]);
            assert_eq!(host.projection.visible_count().get(cx), 1);
        });
    }

    #[gpui::test]
    fn tree_projection_expand_all_uses_latest_tree_order(cx: &mut TestAppContext) {
        struct Host {
            projection: TreeProjection<TreeNode, u64>,
        }

        impl Host {
            fn new(cx: &mut Context<Self>) -> Self {
                init(cx);
                let tree = Signal::new(
                    cx,
                    vec![
                        TreeNode::dir(2, "docs", vec![TreeNode::file(3, "guide.md")]),
                        TreeNode::dir(1, "src", vec![TreeNode::file(4, "main.rs")]),
                    ],
                );
                let tree_for_projection = tree.clone();
                let projection = use_tree_projection(
                    cx,
                    Vec::<u64>::new(),
                    move |cx| tree_for_projection.get(cx),
                    |node| node.id,
                    |node| node.children.clone(),
                );

                Self { projection }
            }
        }

        let entity = cx.update(|cx| cx.new(Host::new));

        cx.update_entity(&entity, |host, cx| {
            assert!(host.projection.expand_all(cx));
        });

        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.projection.expanded_keys(cx), vec![2, 1]);
            assert_eq!(host.projection.visible_keys(cx), vec![2, 3, 1, 4]);
            assert!(host.projection.has_visible_nodes().get(cx));
        });
    }
}
