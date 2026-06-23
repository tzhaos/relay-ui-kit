//! Tree projection - visible tree flattening plus multi-selection.
//!
//! This demo shows how `use_tree_projection(...)` and
//! `use_multi_selection_model(...)` compose into a panel-like GPUI surface:
//!
//! - tree expansion is source-driven and explicit;
//! - visible key order feeds multi-selection automatically;
//! - active and marked rows stay aligned with the latest visible tree.
//!
//! Run with `cargo run -p relay --example tree_projection`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    AnyElement, App, Bounds, Context, Div, InteractiveElement, IntoElement, ParentElement, Render,
    Stateful, Styled, Window, WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_platform::application;
use relay::{
    Memo, MultiSelectionModel, ProjectedTreeNode, ReactiveView, SelectionReconcilePolicy, Signal,
    TreeProjection, init, use_multi_selection_model, use_tree_projection, view::reactive_render,
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
}

impl ReactiveView for TreeProjectionDemo {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let tree_roots = self.tree.get(cx).len();
        let visible_nodes = self.projection.visible_nodes(cx);
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
                            .child(format!("Marked: {marked_count}")),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(toolbar_button("previous", "Previous").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.select_previous(cx);
                        },
                    )))
                    .child(toolbar_button("next", "Next").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.select_next(cx);
                        },
                    )))
                    .child(toolbar_button("extend", "Extend").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.extend_next(cx);
                        },
                    )))
                    .child(toolbar_button("mark", "Toggle mark").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.toggle_active_mark(cx);
                        },
                    )))
                    .child(
                        toolbar_button("toggle", "Toggle folder").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.toggle_active_expansion(cx);
                            },
                        )),
                    )
                    .child(toolbar_button("all", "Select all").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.select_all_visible(cx);
                        },
                    )))
                    .child(toolbar_button("expand", "Expand all").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.expand_all(cx);
                        },
                    )))
                    .child(
                        toolbar_button("collapse", "Collapse all").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.collapse_all(cx);
                            },
                        )),
                    )
                    .child(
                        toolbar_button("reveal", "Reveal leaf").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.reveal_deep_leaf(cx);
                            },
                        )),
                    ),
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
                    .children(visible_nodes.into_iter().map(|node| {
                        let id = *node.key();
                        let label = node.item().label;
                        let depth = node.depth();
                        let has_children = node.has_children();
                        let is_expanded = node.is_expanded();
                        let is_active = self.selection.is_active(cx, id);
                        let is_selected = self.selection.is_selected(cx, id);
                        let selection = self.selection.clone();
                        let projection = self.projection.clone();

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
                            .items_center()
                            .gap_2()
                            .pl(px(10.0 + depth as f32 * 18.0))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0x334155)))
                            .on_click(move |_, _, cx| {
                                let _ = selection.select_only(cx, id);
                                cx.stop_propagation();
                            })
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
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x93c5fd))
                                                .child("active"),
                                        )
                                    })
                                    .when(is_selected && !is_active, |this| {
                                        this.child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x67e8f9))
                                                .child("marked"),
                                        )
                                    }),
                            )
                    })),
            )
            .into_any_element()
    }
}

impl Render for TreeProjectionDemo {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

fn toolbar_button(id: &'static str, label: &'static str) -> Stateful<Div> {
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
