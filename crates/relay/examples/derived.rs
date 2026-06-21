//! Derived — memoized derived values that recompute only when dependencies change.
//!
//! `derived` is a semantic alias for `memo`, emphasizing "derived value". The
//! compute closure runs once on creation and again only when a read signal
//! changes. Multiple reads of the same derived value reuse the cached result.
//!
//! Run with `cargo run -p relay --example derived`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, Bounds, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowBounds, WindowOptions, div, prelude::*,
    px, rgb, size,
};
use gpui_platform::application;
use relay::{Memo, ReactiveAppExt, ReactiveContextExt, Signal, init};

struct DerivedDemo {
    a: Signal<i32>,
    b: Signal<i32>,
    sum: Memo<i32>,
    product: Memo<i32>,
    description: Memo<String>,
}

impl DerivedDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        let a = cx.signal(3);
        let b = cx.signal(4);

        // Simple numeric derivation.
        let sum = {
            let a = a.clone();
            let b = b.clone();
            cx.derived(move |cx| a.get(cx) + b.get(cx))
        };

        let product = {
            let a = a.clone();
            let b = b.clone();
            cx.derived(move |cx| a.get(cx) * b.get(cx))
        };

        // String derivation — recomputes only when a or b changes.
        let description = {
            let a = a.clone();
            let b = b.clone();
            cx.derived(move |cx| format!("{} + {} = {}", a.get(cx), b.get(cx), a.get(cx) + b.get(cx)))
        };

        Self { a, b, sum, product, description }
    }
}

impl Render for DerivedDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.tracked(|cx| {
            let a = self.a.get(cx);
            let b = self.b.get(cx);
            let sum = self.sum.get(cx);
            let product = self.product.get(cx);
            let description = self.description.get(cx);

            div()
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .size_full()
                .bg(rgb(0x202124))
                .text_color(rgb(0xf4f4f5))
                .child(div().text_lg().child("Derived values"))
                .child(div().text_sm().child(format!("a = {a}, b = {b}")))
                .child(div().text_sm().text_color(rgb(0x4ade80)).child(format!("sum (derived) = {sum}")))
                .child(div().text_sm().text_color(rgb(0x60a5fa)).child(format!("product (derived) = {product}")))
                .child(div().text_sm().text_color(rgb(0xa1a1aa)).child(format!("description (derived) = \"{description}\"")))
                .child(
                    div().flex().gap_2()
                        .child(
                            div()
                                .id("inc-a")
                                .px_3().py_2().bg(rgb(0x3b82f6)).rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x2563eb)))
                                .child("a + 1")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.a.update(cx, |v| { *v += 1; true });
                                })),
                        )
                        .child(
                            div()
                                .id("inc-b")
                                .px_3().py_2().bg(rgb(0x8b5cf6)).rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x7c3aed)))
                                .child("b + 1")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.b.update(cx, |v| { *v += 1; true });
                                })),
                        ),
                )
        })
    }
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(360.0), px(280.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(DerivedDemo::new),
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
