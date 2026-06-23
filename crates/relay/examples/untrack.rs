//! Untrack and silent writes — read without subscribing, write without notifying.
//!
//! `untrack` lets you read a signal's value during render without registering
//! a dependency, so the view won't refresh when that signal changes.
//! `set_silent` / `update_silent` write a value without notifying dependents.
//!
//! Run with `cargo run -p relay --example untrack`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, Bounds, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};
use gpui_platform::application;
use relay::{ReactiveAppExt, ReactiveContextExt, Signal, init, untrack};

struct UntrackDemo {
    tracked: Signal<i32>,
    snapshot: Signal<i32>,
    silent_count: Signal<i32>,
}

impl UntrackDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        Self {
            tracked: cx.signal(0),
            snapshot: cx.signal(0),
            silent_count: cx.signal(0),
        }
    }
}

impl Render for UntrackDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.tracked(|cx| {
            // `tracked` is read with dependency tracking — the view refreshes
            // when it changes.
            let tracked_val = self.tracked.get(cx);

            // `snapshot` is read via `untrack` — the view does NOT refresh
            // when snapshot changes. It only shows the value at render time.
            let snapshot_val = untrack(cx, |cx| self.snapshot.get(cx));

            // `silent_count` is read normally (subscribes), but its writes use
            // `set_silent` so incrementing it won't trigger a refresh.
            let silent_val = self.silent_count.get(cx);

            div()
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .size_full()
                .bg(rgb(0x202124))
                .text_color(rgb(0xf4f4f5))
                .child(
                    div()
                        .text_sm()
                        .child(format!("Tracked (subscribed): {tracked_val}")),
                )
                .child(div().text_sm().text_color(rgb(0xa1a1aa)).child(format!(
                    "Snapshot (untracked, stale until other signal changes): {snapshot_val}"
                )))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xfbbf24))
                        .child(format!("Silent count (writes don't notify): {silent_val}")),
                )
                .child(
                    div()
                        .id("inc-tracked")
                        .px_3()
                        .py_2()
                        .bg(rgb(0x3b82f6))
                        .rounded(px(6.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(0x2563eb)))
                        .child("Inc tracked (refreshes view)")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.tracked.update(cx, |v| {
                                *v += 1;
                                true
                            });
                        })),
                )
                .child(
                    div()
                        .id("inc-snapshot")
                        .px_3()
                        .py_2()
                        .bg(rgb(0x27272a))
                        .rounded(px(6.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(0x3f3f46)))
                        .child("Inc snapshot (no refresh — use Inc tracked to see updated value)")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.snapshot.update(cx, |v| {
                                *v += 1;
                                true
                            });
                        })),
                )
                .child(
                    div()
                        .id("inc-silent")
                        .px_3()
                        .py_2()
                        .bg(rgb(0x27272a))
                        .rounded(px(6.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(0x3f3f46)))
                        .child("Inc silent (no refresh — use Inc tracked to see updated value)")
                        .on_click(cx.listener(|this, _, _, _cx| {
                            // set_silent writes without notifying dependents.
                            this.silent_count
                                .set_silent(this.silent_count.get_untracked() + 1);
                        })),
                )
        })
    }
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(440.0), px(280.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(UntrackDemo::new),
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
