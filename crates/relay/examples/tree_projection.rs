//! Tree projection - visible tree flattening plus retained rows.
//!
//! This demo shows how `use_tree_projection(...)`,
//! `use_multi_selection_model(...)`, and `KeyedSubViews` compose into a
//! panel-like GPUI surface:
//!
//! - tree expansion is source-driven and explicit;
//! - visible key order feeds multi-selection automatically;
//! - active and marked rows stay aligned with the latest visible tree;
//! - retained row entities keep local state while sibling branches change.
//!
//! Run with `cargo run -p relay --example tree_projection`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    AnyElement, App, Bounds, Context, Div, InteractiveElement, IntoElement, ParentElement, Render,
    Stateful, Styled, Window, WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_platform::application;
use relay::{
    KeyedSubViews, Memo, MultiSelectionModel, ProjectedTreeNode, ReactiveView,
    SelectionReconcilePolicy, Signal, TreeProjection, init, use_multi_selection_model,
    use_tree_projection, view::reactive_render,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileNode {
    id: u64,
    label: &'static str,
    children: Vec<FileNode>,
}

impl FileNode {
    fn file(id: u64, label: &'static str) -> Self {
        Self {
            id,
            label,
            children: Vec::new(),
        }
    }

    fn dir(id: u64, label: &'static str, children: Vec<FileNode>) -> Self {
        Self {
            id,
            label,
            children,
        }
    }
}

struct TreeProjectionDemo {
    tree: Signal<Vec<FileNode>>,
    projection: TreeProjection<FileNode, u64>,
    selection: MultiSelectionModel<u64>,
    active_node: Memo<Option<ProjectedTreeNode<FileNode, u64>>>,
    selected_nodes: Memo<Vec<ProjectedTreeNode<FileNode, u64>>>,
    rows: KeyedSubViews<u64, TreeRow>,
    last_action: Signal<String>,
}

impl TreeProjectionDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);

        let tree = Signal::new(cx, demo_tree());
        let tree_for_projection = tree.clone();
        let projection = use_tree_projection(
            cx,
            [1, 2],
            move |cx| tree_for_projection.get(cx),
            |node| node.id,
            |node| node.children.clone(),
        );
        let projection_for_selection = projection.clone();
        let selection = use_multi_selection_model(
            cx,
            Some(3),
            [3],
            move |cx| projection_for_selection.visible_keys(cx),
            SelectionReconcilePolicy::SelectFirst,
        );
        let active_node =
            selection.active_from_signal(cx, projection.visible_nodes_signal(), |node| *node.key());
        let selected_nodes =
            selection.selected_items_from_signal(cx, projection.visible_nodes_signal(), |node| {
                *node.key()
            });

        Self {
            tree,
            projection,
            selection,
            active_node,
            selected_nodes,
            rows: KeyedSubViews::new(),
            last_action: Signal::new(cx, "ready".to_string()),
        }
    }

    fn select_next(&self, cx: &mut App) {
        if self.selection.select_next_only(cx) {
            self.last_action.set(cx, "moved to next row".to_string());
        }
    }

    fn select_previous(&self, cx: &mut App) {
        if self.selection.select_previous_only(cx) {
            self.last_action
                .set(cx, "moved to previous row".to_string());
        }
    }

    fn extend_next(&self, cx: &mut App) {
        if self.selection.extend_next(cx) {
            self.last_action
                .set(cx, "extended selection downward".to_string());
        }
    }

    fn toggle_active_mark(&self, cx: &mut App) {
        let Some(active) = self.selection.active_untracked() else {
            return;
        };
        if self.selection.toggle(cx, active) {
            self.last_action
                .set(cx, "toggled the active row in the marked set".to_string());
        }
    }

    fn toggle_active_expansion(&self, cx: &mut App) {
        let Some(active) = self.selection.active_untracked() else {
            return;
        };
        if self.projection.toggle(cx, active) {
            self.last_action
                .set(cx, "toggled the active row expansion".to_string());
        }
    }

    fn select_all_visible(&self, cx: &mut App) {
        if self.selection.select_all(cx) {
            self.last_action
                .set(cx, "selected every visible row".to_string());
        }
    }

    fn expand_all(&self, cx: &mut App) {
        if self.projection.expand_all(cx) {
            self.last_action
                .set(cx, "expanded every folder".to_string());
        }
    }

    fn collapse_all(&self, cx: &mut App) {
        if self.projection.collapse_all(cx) {
            self.last_action
                .set(cx, "collapsed every folder".to_string());
        }
    }

    fn reveal_deep_leaf(&self, cx: &mut App) {
        if self.projection.reveal(cx, 10) {
            self.last_action
                .set(cx, "revealed relay_uikit/button.rs".to_string());
        }
        let _ = self.selection.select_only(cx, 10);
    }

    #[cfg(test)]
    fn visible_keys(&self, cx: &App) -> Vec<u64> {
        self.projection.visible_keys(cx)
    }
}

impl ReactiveView for TreeProjectionDemo {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let tree_roots = self.tree.get(cx).len();
        let visible_nodes = self.projection.visible_nodes(cx);
        let row_count = visible_nodes.len();
        let selected_labels = self.selected_nodes.read(cx, |nodes| {
            nodes
                .iter()
                .map(|node| node.item().label)
                .collect::<Vec<_>>()
                .join(", ")
        });
        let active_label = self
            .active_node
            .get(cx)
            .map(|node| node.item().label)
            .unwrap_or("None");
        let marked_count = self.selection.selection_count().get(cx);
        let visible_count = self.projection.visible_count().get(cx);
        let last_action = self.last_action.get(cx);
        let selection_for_rows = self.selection.clone();
        let projection_for_rows = self.projection.clone();

        self.rows.sync(
            cx,
            visible_nodes,
            |node| *node.key(),
            move |node, cx| {
                TreeRow::new(
                    node,
                    selection_for_rows.clone(),
                    projection_for_rows.clone(),
                    cx,
                )
            },
            |node, row, _cx| row.update_node(node),
        );

        div()
            .id("tree-projection-demo")
            .size_full()
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x18181b))
            .text_color(rgb(0xf8fafc))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(div().text_lg().child("Tree projection demo"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xa1a1aa))
                            .child(format!("Roots: {tree_roots}")),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xa1a1aa))
                            .child(format!("Visible: {visible_count}")),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xa1a1aa))
                            .child(format!("Rows: {row_count}")),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xa1a1aa))
                            .child(format!("Marked: {marked_count}")),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(button_pill("previous", "Previous").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.select_previous(cx);
                        },
                    )))
                    .child(
                        button_pill("next", "Next").on_click(cx.listener(|this, _, _, cx| {
                            this.select_next(cx);
                        })),
                    )
                    .child(button_pill("extend", "Extend").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.extend_next(cx);
                        },
                    )))
                    .child(button_pill("mark", "Toggle mark").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.toggle_active_mark(cx);
                        },
                    )))
                    .child(button_pill("toggle", "Toggle folder").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.toggle_active_expansion(cx);
                        },
                    )))
                    .child(button_pill("all", "Select all").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.select_all_visible(cx);
                        },
                    )))
                    .child(button_pill("expand", "Expand all").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.expand_all(cx);
                        },
                    )))
                    .child(
                        button_pill("collapse", "Collapse all").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.collapse_all(cx);
                            },
                        )),
                    )
                    .child(button_pill("reveal", "Reveal leaf").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.reveal_deep_leaf(cx);
                        },
                    ))),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0xa1a1aa))
                    .child(format!("Active: {active_label}")),
            )
            .child(div().text_xs().text_color(rgb(0xa1a1aa)).child(format!(
                "Marked rows: {}",
                if selected_labels.is_empty() {
                    "None".to_string()
                } else {
                    selected_labels
                }
            )))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0xa1a1aa))
                    .child(format!("Last action: {last_action}")),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .children(self.rows.cached(gpui::StyleRefinement::default().w_full())),
            )
            .into_any_element()
    }
}

impl Render for TreeProjectionDemo {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

struct TreeRow {
    node: ProjectedTreeNode<FileNode, u64>,
    selection: MultiSelectionModel<u64>,
    projection: TreeProjection<FileNode, u64>,
    details_open: Signal<bool>,
}

impl TreeRow {
    fn new(
        node: &ProjectedTreeNode<FileNode, u64>,
        selection: MultiSelectionModel<u64>,
        projection: TreeProjection<FileNode, u64>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            node: node.clone(),
            selection,
            projection,
            details_open: Signal::new(cx, false),
        }
    }

    fn update_node(&mut self, node: &ProjectedTreeNode<FileNode, u64>) -> bool {
        if self.node == *node {
            false
        } else {
            self.node = node.clone();
            true
        }
    }

    #[cfg(test)]
    fn toggle_details(&self, cx: &mut App) {
        self.details_open.update(cx, |details_open| {
            *details_open = !*details_open;
            true
        });
    }
}

impl ReactiveView for TreeRow {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let id = *self.node.key();
        let label = self.node.item().label;
        let depth = self.node.depth();
        let has_children = self.node.has_children();
        let is_expanded = self.node.is_expanded();
        let details_open = self.details_open.get(cx);
        let is_active = self.selection.is_active(cx, id);
        let is_selected = self.selection.is_selected(cx, id);
        let selection = self.selection.clone();
        let projection = self.projection.clone();
        let details_signal = self.details_open.clone();

        div()
            .id(format!("tree-row-{id}"))
            .px_2()
            .py_2()
            .rounded(px(6.0))
            .border_1()
            .border_color(if is_active {
                rgb(0x60a5fa)
            } else if is_selected {
                rgb(0x38bdf8)
            } else {
                rgb(0x3f3f46)
            })
            .bg(if is_active {
                rgb(0x1e3a8a)
            } else if is_selected {
                rgb(0x164e63)
            } else {
                rgb(0x27272a)
            })
            .flex()
            .flex_col()
            .gap_2()
            .cursor_pointer()
            .hover(|style| style.bg(rgb(0x334155)))
            .on_click(move |_, _, cx| {
                let _ = selection.select_only(cx, id);
                cx.stop_propagation();
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .pl(px(10.0 + depth as f32 * 18.0))
                    .child(if has_children {
                        div()
                            .id(format!("toggle-{id}"))
                            .w(px(18.0))
                            .text_xs()
                            .text_color(rgb(0xcbd5e1))
                            .cursor_pointer()
                            .child(if is_expanded { "v" } else { ">" })
                            .on_click(move |_, _, cx| {
                                let _ = projection.toggle(cx, id);
                                cx.stop_propagation();
                            })
                            .into_any_element()
                    } else {
                        div()
                            .w(px(18.0))
                            .text_xs()
                            .text_color(rgb(0x64748b))
                            .child("-")
                            .into_any_element()
                    })
                    .child(
                        div()
                            .min_w_0()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().truncate().child(label))
                            .when(is_active, |this| {
                                this.child(
                                    div().text_xs().text_color(rgb(0x93c5fd)).child("active"),
                                )
                            })
                            .when(is_selected && !is_active, |this| {
                                this.child(
                                    div().text_xs().text_color(rgb(0x67e8f9)).child("marked"),
                                )
                            }),
                    )
                    .child(
                        button_pill("details", if details_open { "Hide" } else { "Details" })
                            .on_click(move |_, _, cx| {
                                details_signal.update(cx, |details_open| {
                                    *details_open = !*details_open;
                                    true
                                });
                                cx.stop_propagation();
                            }),
                    ),
            )
            .when(details_open, |this| {
                this.child(
                    div()
                        .pl(px(28.0 + depth as f32 * 18.0))
                        .text_xs()
                        .text_color(rgb(0xcbd5e1))
                        .child(format!(
                            "id: {id}, depth: {depth}, kind: {}",
                            if has_children { "dir" } else { "file" }
                        )),
                )
            })
            .into_any_element()
    }
}

impl Render for TreeRow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

fn button_pill(id: &'static str, label: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .px_2()
        .py_1()
        .rounded(px(4.0))
        .bg(rgb(0x3b82f6))
        .hover(|style| style.bg(rgb(0x2563eb)))
        .cursor_pointer()
        .text_xs()
        .child(label)
}

fn demo_tree() -> Vec<FileNode> {
    vec![FileNode::dir(
        1,
        "workspace",
        vec![
            FileNode::dir(
                2,
                "relay",
                vec![
                    FileNode::file(3, "Cargo.toml"),
                    FileNode::dir(
                        4,
                        "src",
                        vec![
                            FileNode::file(5, "lib.rs"),
                            FileNode::file(6, "query.rs"),
                            FileNode::file(7, "mutation.rs"),
                        ],
                    ),
                    FileNode::dir(
                        8,
                        "relay_uikit",
                        vec![
                            FileNode::file(9, "theme.rs"),
                            FileNode::file(10, "button.rs"),
                        ],
                    ),
                ],
            ),
            FileNode::dir(
                11,
                "docs",
                vec![
                    FileNode::file(12, "relay-v2-rfc.md"),
                    FileNode::file(13, "relay-v2-implementation-plan.md"),
                ],
            ),
        ],
    )]
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(760.0), px(520.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(TreeProjectionDemo::new),
        );
        cx.activate(true);
    });
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run_example();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run_example();
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use gpui::{Entity, EntityId, TestApp};

    use super::*;

    fn row_ids(rows: &KeyedSubViews<u64, TreeRow>) -> Vec<(u64, EntityId)> {
        rows.keyed_iter()
            .map(|(key, view)| (*key, view.entity().entity_id()))
            .collect()
    }

    fn row_entity(surface: &TreeProjectionDemo, key: u64) -> Entity<TreeRow> {
        match surface.rows.get(&key) {
            Some(row) => row.clone_entity(),
            None => panic!("missing tree row {key}"),
        }
    }

    #[test]
    fn tree_projection_demo_reuses_existing_rows_when_branch_expands() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| TreeProjectionDemo::new(cx));
        let root = window.root();

        window.draw();
        let initial_rows = app.update_entity(&root, |surface, _cx| row_ids(&surface.rows));
        let initial_by_key = initial_rows.into_iter().collect::<HashMap<_, _>>();

        app.update_entity(&root, |surface, cx| {
            assert!(surface.projection.toggle(cx, 11));
        });
        window.draw();

        let updated_rows = app.update_entity(&root, |surface, _cx| row_ids(&surface.rows));
        assert_eq!(
            updated_rows.iter().map(|(key, _)| *key).collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 8, 11, 12, 13]
        );

        for key in [1_u64, 2, 3, 4, 8, 11] {
            let entity_id = updated_rows
                .iter()
                .find_map(|(row_key, entity_id)| (*row_key == key).then_some(*entity_id))
                .unwrap_or_else(|| panic!("missing row id for {key}"));
            assert_eq!(entity_id, initial_by_key[&key]);
        }
    }

    #[test]
    fn tree_projection_demo_row_state_survives_sibling_branch_expansion() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| TreeProjectionDemo::new(cx));
        let root = window.root();

        window.draw();
        let row = app.update_entity(&root, |surface, _cx| row_entity(surface, 3));
        app.update_entity(&row, |row, cx| {
            row.toggle_details(cx);
        });

        app.update_entity(&root, |surface, cx| {
            assert!(surface.projection.toggle(cx, 11));
        });
        window.draw();

        let row_after = app.update_entity(&root, |surface, _cx| row_entity(surface, 3));
        let details_open =
            app.update_entity(&row_after, |row, _cx| row.details_open.get_untracked());
        assert_eq!(row.entity_id(), row_after.entity_id());
        assert!(details_open);
    }

    #[test]
    fn tree_projection_demo_reveal_selects_deep_leaf_and_expands_ancestors() {
        let mut app = TestApp::new();
        let window = app.open_window(|_, cx| TreeProjectionDemo::new(cx));
        let root = window.root();

        app.update_entity(&root, |surface, cx| {
            surface.reveal_deep_leaf(cx);
        });

        let (active, selected, visible_keys) = app.update_entity(&root, |surface, cx| {
            (
                surface.selection.active_untracked(),
                surface.selection.selected_keys_untracked(),
                surface.visible_keys(cx),
            )
        });

        assert_eq!(active, Some(10));
        assert_eq!(selected, vec![10]);
        assert_eq!(visible_keys, vec![1, 2, 3, 4, 8, 9, 10, 11]);
    }
}
