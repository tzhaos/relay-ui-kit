//! Effect — reactive side effects that re-run when dependencies change.
//!
//! An `Effect` runs its callback immediately to discover dependencies, then
//! re-runs whenever any of those dependencies change. `effect_in` scopes the
//! effect to a GPUI entity so it is automatically disposed when the entity is
//! released.
//!
//! Run with `cargo run -p relay --example effect`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, Bounds, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowBounds, WindowOptions, div, prelude::*,
    px, rgb, size,
};
use gpui_platform::application;
use relay::{Effect, ReactiveAppExt, ReactiveContextExt, Signal, effect_in, init};

struct EffectDemo {
    value: Signal<i32>,
    log: Signal<Vec<String>>,
    _log_effect: Effect,
}

impl EffectDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        let value = cx.signal(0);
        let log = cx.signal(Vec::new());

        // effect_in scopes the effect to this entity. When `value` changes,
        // the effect re-runs and appends a log entry.
        let log_effect = {
            let value = value.clone();
            let log = log.clone();
            effect_in(cx, move |cx| {
                let current = value.get(cx);
                log.update(cx, |entries| {
                    entries.push(format!("value changed to {current}"));
                    true
                });
            })
        };

        // Seed an initial log entry (the effect already ran once on creation).
        log.update(cx, |entries| {
            entries.push("Effect demo started".into());
            true
        });

        Self {
            value,
            log,
            _log_effect: log_effect,
        }
    }

    fn increment(&self, cx: &mut App) {
        self.value.update(cx, |v| {
            *v += 1;
            true
        });
    }
}

impl Render for EffectDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.tracked(|cx| {
            let value = self.value.get(cx);
            let entries = self.log.get(cx);

            let mut log_view = div().flex().flex_col().gap_1();
            for entry in entries.iter().rev().take(5) {
                log_view = log_view.child(
                    div()
                        .text_xs()
                        .text_color(rgb(0xa1a1aa))
                        .child(format!("• {entry}")),
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
                .child(div().text_lg().child(format!("Value: {value}")))
                .child(
                    div()
                        .id("increment")
                        .px_3().py_2().bg(rgb(0x3b82f6)).rounded(px(6.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(0x2563eb)))
                        .child("Increment (triggers effect)")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.increment(cx);
                        })),
                )
                .child(div().text_sm().text_color(rgb(0xa1a1aa)).child("Effect log (last 5):"))
                .child(log_view)
        })
    }
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(320.0), px(300.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(EffectDemo::new),
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
