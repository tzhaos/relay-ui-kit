//! Ordered selection - source-driven selection with automatic reconcile.
//!
//! `use_ordered_selection_model(...)` packages the common list/picker shape:
//! a tracked ordered key source drives reconcile, while `select_next()` and
//! `select_previous()` reuse that latest visible order without caller-side key
//! plumbing.
//!
//! Run with `cargo run -p relay --example ordered_selection`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    AnyElement, App, Bounds, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};
use gpui_platform::application;
use relay::{
    Memo, OrderedSelectionModel, ReactiveView, SelectionReconcilePolicy, Signal, init,
    use_ordered_selection_model, view::reactive_render,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Item {
    id: u64,
    label: &'static str,
}

impl Item {
    fn new(id: u64, label: &'static str) -> Self {
        Self { id, label }
    }
}

struct OrderedSelectionDemo {
    filter: Signal<&'static str>,
    items: Signal<Vec<Item>>,
    selection: OrderedSelectionModel<u64>,
    selected_item: Memo<Option<Item>>,
}

impl OrderedSelectionDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        let filter = Signal::new(cx, "all");
        let items = Signal::new(
            cx,
            vec![
                Item::new(1, "alpha"),
                Item::new(2, "beta"),
                Item::new(3, "alpine"),
                Item::new(4, "gamma"),
            ],
        );

        let filter_for_selection = filter.clone();
        let items_for_selection = items.clone();
        let selection = use_ordered_selection_model(
            cx,
            Some(3),
            move |cx| {
                let filter = filter_for_selection.get(cx);
                items_for_selection.read(cx, |items| {
                    items
                        .iter()
                        .filter(|item| matches_filter(item, filter))
                        .map(|item| item.id)
                        .collect()
                })
            },
            SelectionReconcilePolicy::SelectFirst,
        );
        let selected_item = selection.selected_from_signal(cx, &items, |item| item.id);

        Self {
            filter,
            items,
            selection,
            selected_item,
        }
    }

    fn cycle_filter(&self, cx: &mut App) {
        let next = match self.filter.get_untracked() {
            "all" => "al",
            "al" => "be",
            _ => "all",
        };
        self.filter.set(cx, next);
    }

    fn select_next(&self, cx: &mut App) {
        let _ = self.selection.select_next(cx);
    }

    fn select_previous(&self, cx: &mut App) {
        let _ = self.selection.select_previous(cx);
    }
}

impl ReactiveView for OrderedSelectionDemo {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let filter = self.filter.get(cx);
        let selected = self.selected_item.get(cx).map(|item| item.label);
        let visible = self.items.read(cx, |items| {
            items
                .iter()
                .filter(|item| matches_filter(item, filter))
                .map(|item| item.label)
                .collect::<Vec<_>>()
                .join(", ")
        });

        div()
            .size_full()
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x202124))
            .text_color(rgb(0xf4f4f5))
            .child(div().text_lg().child("Ordered selection demo"))
            .child(div().text_sm().child(format!("Filter: {filter}")))
            .child(div().text_sm().child(format!("Visible items: {visible}")))
            .child(
                div().text_sm().text_color(rgb(0x60a5fa)).child(format!(
                    "Selected item: {}",
                    selected.unwrap_or("none")
                )),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0xa1a1aa))
                    .child("Changing the filter automatically reconciles selection. Next/previous always use the latest visible order."),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        demo_button("cycle-filter", "Cycle filter").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.cycle_filter(cx);
                            },
                        )),
                    )
                    .child(
                        demo_button("previous-item", "Previous").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.select_previous(cx);
                            },
                        )),
                    )
                    .child(
                        demo_button("next-item", "Next").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.select_next(cx);
                            },
                        )),
                    ),
            )
            .into_any_element()
    }
}

impl Render for OrderedSelectionDemo {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

fn demo_button(id: &'static str, label: &'static str) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .px_3()
        .py_2()
        .rounded(px(6.0))
        .bg(rgb(0x3b82f6))
        .hover(|style| style.bg(rgb(0x2563eb)))
        .cursor_pointer()
        .text_xs()
        .child(label)
}

fn matches_filter(item: &Item, filter: &str) -> bool {
    match filter {
        "all" => true,
        other => item.label.contains(other),
    }
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(620.0), px(240.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(OrderedSelectionDemo::new),
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
    use gpui::TestApp;

    use super::*;

    #[test]
    fn ordered_selection_example_reconciles_selection_when_filter_changes() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| OrderedSelectionDemo::new(cx));
        let root = window.root();
        window.draw();

        let initial = app.update_entity(&root, |demo, cx| {
            demo.selected_item.get(cx).map(|item| item.label)
        });
        assert_eq!(initial, Some("alpine"));

        app.update_entity(&root, |demo, cx| {
            demo.filter.set(cx, "be");
        });

        let filtered = app.update_entity(&root, |demo, cx| {
            (
                demo.selection.get(cx),
                demo.selected_item.get(cx).map(|item| item.label),
            )
        });
        assert_eq!(filtered, (Some(2), Some("beta")));
    }
}
