//! SignalVec — reactive list operations on `Signal<Vec<T>>`.
//!
//! `SignalVecExt` provides `push`, `extend`, `insert`, `remove`, `retain`,
//! `clear`, and `set_all` for `Signal<Vec<T>>`. Each mutation notifies
//! dependents through the normal signal path, so any view reading the list
//! refreshes automatically.
//!
//! Run with `cargo run -p relay --example signal_vec`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, Bounds, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};
use gpui_platform::application;
use relay::{ReactiveAppExt, ReactiveContextExt, Signal, SignalVecExt, init};

struct SignalVecDemo {
    items: Signal<Vec<String>>,
}

impl SignalVecDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        Self {
            items: cx.signal(vec!["First item".into(), "Second item".into()]),
        }
    }
}

impl Render for SignalVecDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.tracked(|cx| {
            let items = self.items.get(cx);

            let mut list = div().flex().flex_col().gap_1();
            for (i, item) in items.iter().enumerate() {
                let item_label = item.clone();
                list = list.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .py_2()
                        .bg(rgb(0x27272a))
                        .rounded(px(6.0))
                        .child(div().text_sm().flex_1().child(format!("{i}. {item_label}")))
                        .child(
                            div()
                                .id(("remove", i))
                                .px_2()
                                .py_1()
                                .bg(rgb(0xef4444))
                                .rounded(px(4.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0xdc2626)))
                                .text_xs()
                                .child("Remove")
                                .on_click({
                                    let items = self.items.clone();
                                    let target = item_label.clone();
                                    move |_, _, cx| {
                                        items.retain(cx, |x| x != &target);
                                    }
                                }),
                        ),
                );
            }

            div()
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .size_full()
                .bg(rgb(0x202124))
                .text_color(rgb(0xf4f4f5))
                .child(div().text_lg().child(format!("Items ({})", items.len())))
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .id("add")
                                .px_3()
                                .py_2()
                                .bg(rgb(0x3b82f6))
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x2563eb)))
                                .child("Push item")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    let count = this.items.read(cx, |v| v.len());
                                    this.items.push(cx, format!("Item {}", count + 1));
                                })),
                        )
                        .child(
                            div()
                                .id("add-many")
                                .px_3()
                                .py_2()
                                .bg(rgb(0x1d4ed8))
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x1e40af)))
                                .child("Push 3")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    let count = this.items.read(cx, |v| v.len());
                                    this.items.extend(
                                        cx,
                                        (1..=3).map(|offset| format!("Item {}", count + offset)),
                                    );
                                })),
                        )
                        .child(
                            div()
                                .id("clear")
                                .px_3()
                                .py_2()
                                .bg(rgb(0x27272a))
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x3f3f46)))
                                .child("Clear all")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.items.clear(cx);
                                })),
                        ),
                )
                .child(list)
        })
    }
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(360.0), px(400.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(SignalVecDemo::new),
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
