use std::hash::Hash;

use crate::{
    component_prelude::*,
    icon::{Icon, IconName, IconSize},
    interaction::SharedActionHandler,
};

use super::ListItem;

/// One flattened tree node rendered by [`TreeView`].
#[derive(Clone, PartialEq)]
pub struct TreeNode<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    key: K,
    icon: IconName,
    label: String,
    depth: usize,
    expanded: Option<bool>,
    selected: bool,
}

impl<K> TreeNode<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    /// Create a node with a stable key, leading icon, and visible label.
    pub fn new(key: K, icon: IconName, label: impl Into<String>) -> Self {
        Self {
            key,
            icon,
            label: label.into(),
            depth: 0,
            expanded: None,
            selected: false,
        }
    }

    /// Indent the node to represent hierarchical depth.
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    /// Mark the node as expandable and set its current open state.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    /// Render the node in the selected state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

/// Flat tree surface with host-owned selection and disclosure behavior.
///
/// `TreeView` expects the host to provide already-flattened nodes in display
/// order. Nodes with [`TreeNode::expanded`] set become disclosure rows and will
/// use [`TreeView::on_toggle`] when activated. Leaf rows use
/// [`TreeView::on_select`] instead.
#[derive(IntoElement)]
pub struct TreeView<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    id: ElementId,
    nodes: Vec<TreeNode<K>>,
    on_select: Option<SharedActionHandler<K>>,
    on_toggle: Option<SharedActionHandler<K>>,
}

impl<K> TreeView<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    /// Create a tree from the current visible node snapshot.
    pub fn new(id: impl Into<ElementId>, nodes: Vec<TreeNode<K>>) -> Self {
        Self {
            id: id.into(),
            nodes,
            on_select: None,
            on_toggle: None,
        }
    }

    /// Observe activation of non-expandable rows.
    pub fn on_select(mut self, handler: impl Fn(K, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(std::rc::Rc::new(handler));
        self
    }

    /// Observe activation of expandable rows.
    pub fn on_toggle(mut self, handler: impl Fn(K, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(std::rc::Rc::new(handler));
        self
    }
}

impl<K> RenderOnce for TreeView<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();

        div()
            .id(self.id)
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .children(self.nodes.into_iter().enumerate().map(|(index, node)| {
                let key = node.key.clone();
                let mut row = ListItem::new(("tree-node", index))
                    .height(px(space::ROW_SM))
                    .indent(node.depth, 14.0)
                    .selected(node.selected)
                    .start_slot(tree_node_icon(theme, node.icon, node.expanded))
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .truncate()
                            .text_sm()
                            .text_color(if node.selected {
                                theme.text
                            } else {
                                theme.text_secondary
                            })
                            .child(node.label),
                    );

                if let Some(on_toggle) = self.on_toggle.clone().filter(|_| node.expanded.is_some())
                {
                    row = row.on_click(move |_event, window, cx| {
                        on_toggle(key.clone(), window, cx);
                    });
                } else if let Some(on_select) = self.on_select.clone() {
                    row = row.on_click(move |_event, window, cx| {
                        on_select(key.clone(), window, cx);
                    });
                }

                row
            }))
    }
}

fn tree_node_icon(theme: crate::Theme, icon: IconName, expanded: Option<bool>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_1()
        .child(
            div()
                .w(px(14.0))
                .flex()
                .items_center()
                .justify_center()
                .when_some(expanded, |this, expanded| {
                    this.child(
                        Icon::new(if expanded {
                            IconName::ChevronDown
                        } else {
                            IconName::ChevronRight
                        })
                        .size(IconSize::XSmall)
                        .color(theme.text_muted),
                    )
                }),
        )
        .child(
            Icon::new(icon)
                .size(IconSize::Small)
                .color(theme.text_muted),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_node_expanded_marks_node_as_expandable() {
        let node = TreeNode::new("src", IconName::Folder, "src").expanded(true);

        assert_eq!(node.expanded, Some(true));
    }

    #[test]
    fn tree_view_starts_without_toggle_handler() {
        let tree: TreeView<&'static str> = TreeView::new("tree", vec![]);

        assert!(tree.on_toggle.is_none());
    }
}
