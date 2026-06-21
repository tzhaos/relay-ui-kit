//! Reactive struct — field-level reactivity via `#[derive(Reactive)]`.
//!
//! The `Reactive` derive macro transforms a plain struct into a reactive
//! wrapper. By default, each field is wrapped in `Signal<T>`. Fields marked
//! `#[reactive(nested)]` use their own generated reactive wrapper, so nested
//! fields keep independent dependency tracking.
//!
//! Run with `cargo run -p relay --example reactive_struct`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, Bounds, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};
use gpui_platform::application;
use relay::{Reactive, ReactiveContextExt, init};

/// A plain nested struct — the derive macro generates `ReactiveCounterMeta`.
#[derive(Clone, PartialEq, Eq, Reactive)]
struct CounterMeta {
    label: String,
    owner: String,
}

/// A plain struct — the derive macro generates `ReactiveCounter`.
#[derive(Reactive)]
#[allow(dead_code)]
struct Counter {
    count: i32,
    #[reactive(nested)]
    meta: CounterMeta,
}

struct ReactiveStructDemo {
    counter: ReactiveCounter,
}

impl ReactiveStructDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        let counter = ReactiveCounter::from(
            cx,
            Counter {
                count: 0,
                meta: CounterMeta {
                    label: "Clicks".into(),
                    owner: "Relay".into(),
                },
            },
        );
        Self { counter }
    }
}

impl Render for ReactiveStructDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.tracked(|cx| {
            // Each field is read independently — only the specific field's
            // signal is tracked. Nested fields keep the same granularity.
            let count = self.counter.get_count(cx);
            let label = self.counter.reactive_meta().get_label(cx);
            let owner = self.counter.reactive_meta().get_owner(cx);

            div()
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .size_full()
                .bg(rgb(0x202124))
                .text_color(rgb(0xf4f4f5))
                .child(div().text_lg().child("Reactive struct demo"))
                .child(div().text_sm().text_color(rgb(0xa1a1aa)).child(
                    "Each field is a Signal<T> — field-level reactivity \
                             without manual signal creation. Nested structs can \
                             be marked with #[reactive(nested)].",
                ))
                .child(div().text_xl().child(format!("{label}: {count}")))
                .child(div().text_sm().text_color(rgb(0xa1a1aa)).child(owner))
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .id("inc-count")
                                .px_3()
                                .py_2()
                                .bg(rgb(0x3b82f6))
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x2563eb)))
                                .child("Inc count")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.counter.update_count(cx, |v| {
                                        *v += 1;
                                        true
                                    });
                                })),
                        )
                        .child(
                            div()
                                .id("change-label")
                                .px_3()
                                .py_2()
                                .bg(rgb(0x8b5cf6))
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x7c3aed)))
                                .child("Change label")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    let labels = ["Clicks", "Taps", "Presses", "Interactions"];
                                    let meta = this.counter.reactive_meta();
                                    let current = meta.get_label(cx);
                                    let idx =
                                        labels.iter().position(|l| *l == current).unwrap_or(0);
                                    let next = labels[(idx + 1) % labels.len()];
                                    meta.set_label(cx, next.into());
                                })),
                        ),
                )
        })
    }
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(360.0), px(240.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(ReactiveStructDemo::new),
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
