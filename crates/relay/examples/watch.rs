//! Watch — declarative side effects with explicit dependency tracking.
//!
//! `watch(cx, sources, react)` separates dependency declaration (`sources`)
//! from the side effect (`react`). The `sources` closure reads signals to
//! register dependencies; `react` runs the side effect when any dependency
//! changes. This is the Vue `watch` equivalent.
//!
//! Run with `cargo run -p relay --example watch`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, Bounds, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowBounds, WindowOptions, div, prelude::*,
    px, rgb, size,
};
use gpui_platform::application;
use relay::{ReactiveAppExt, ReactiveContextExt, Signal, init};

struct WatchDemo {
    name: Signal<String>,
    age: Signal<i32>,
    greeting: Signal<String>,
    age_log: Signal<String>,
}

impl WatchDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        let name = cx.signal("Alice".into());
        let age = cx.signal(30);
        let greeting = cx.signal(String::new());
        let age_log = cx.signal("No age changes yet".into());

        // Watch 1: when `name` changes, derive a greeting.
        {
            let name = name.clone();
            let name_for_react = name.clone();
            let greeting = greeting.clone();
            let _ = cx.watch(
                move |cx| { let _ = name.get(cx); },
                move |cx| {
                    let n = name_for_react.get(cx);
                    greeting.set(cx, format!("Hello, {n}!"));
                },
            );
        }

        // Watch 2: when `age` changes, log the transition.
        {
            let age = age.clone();
            let age_for_react = age.clone();
            let age_log = age_log.clone();
            let _ = cx.watch(
                move |cx| { let _ = age.get(cx); },
                move |cx| {
                    let a = age_for_react.get(cx);
                    age_log.set(cx, format!("Age changed to {a}"));
                },
            );
        }

        // Seed the initial greeting.
        greeting.set(cx, format!("Hello, {}!", name.get_untracked()));

        Self { name, age, greeting, age_log }
    }
}

impl Render for WatchDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.tracked(|cx| {
            let name = self.name.get(cx);
            let age = self.age.get(cx);
            let greeting = self.greeting.get(cx);
            let age_log = self.age_log.get(cx);

            div()
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .size_full()
                .bg(rgb(0x202124))
                .text_color(rgb(0xf4f4f5))
                .child(div().text_lg().child("Watch demo"))
                .child(div().text_sm().child(format!("Name: {name}")))
                .child(div().text_sm().child(format!("Age: {age}")))
                .child(div().text_sm().text_color(rgb(0x4ade80)).child(format!("Greeting (derived by watch): \"{greeting}\"")))
                .child(div().text_xs().text_color(rgb(0xa1a1aa)).child(format!("Age log (derived by watch): {age_log}")))
                .child(
                    div().flex().gap_2()
                        .child(
                            div()
                                .id("change-name")
                                .px_3().py_2().bg(rgb(0x3b82f6)).rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x2563eb)))
                                .child("Change name")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    let names = ["Alice", "Bob", "Carol", "Dave"];
                                    let current = this.name.get_untracked();
                                    let idx = names.iter().position(|n| *n == current).unwrap_or(0);
                                    let next = names[(idx + 1) % names.len()];
                                    this.name.set(cx, next.into());
                                })),
                        )
                        .child(
                            div()
                                .id("inc-age")
                                .px_3().py_2().bg(rgb(0x8b5cf6)).rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x7c3aed)))
                                .child("Age + 1")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.age.update(cx, |v| { *v += 1; true });
                                })),
                        ),
                )
        })
    }
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(400.0), px(280.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(WatchDemo::new),
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
