//! Component-internal hooks — stateful RenderOnce components via `use_signal`.
//!
//! GPUI's `RenderOnce` components are normally stateless — each render
//! consumes a new instance. But `Window::use_signal` (backed by
//! `use_keyed_state`) lets a `RenderOnce` component own persistent state
//! across renders, as long as it renders in the same position.
//!
//! This is the React `useState` / Solid `createSignal` equivalent for GPUI
//! components. No `Binding` or host callback needed — the component manages
//! its own state internally.
//!
//! Run with `cargo run -p relay --example component_hooks`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, Bounds, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowBounds, WindowOptions, div, prelude::*,
    px, rgb, size,
};
use gpui_platform::application;
use relay::{ReactiveContextExt, WindowSignalExt, init};

/// A self-contained counter component that owns its state via `use_signal`.
///
/// No external `Signal` or `Binding` is passed in — the component calls
/// `window.use_signal` during render to get a persistent signal.
struct StatefulCounter;

impl StatefulCounter {
    /// Render a stateful counter. Must be called during the layout phase
    /// (inside a view's `render` method), as `use_signal` requires a valid
    /// element state context.
    fn render_component(
        window: &mut Window,
        cx: &mut App,
        id: &'static str,
    ) -> impl IntoElement + use<> {
        // use_signal persists across renders as long as this component keeps
        // rendering at the same position (keyed by `id`).
        let count = window.use_signal(id, cx, || 0);
        let count_val = count.get(cx);
        let count_for_click = count.clone();

        div()
            .id(id)
            .flex()
            .items_center()
            .gap_2()
            .px_4()
            .py_2()
            .bg(rgb(0x3b82f6))
            .rounded(px(6.0))
            .cursor_pointer()
            .hover(|s| s.bg(rgb(0x2563eb)))
            .text_color(rgb(0xf4f4f5))
            .child(format!("Count: {count_val}"))
            .on_click(move |_, _, cx| {
                count_for_click.update(cx, |v| {
                    *v += 1;
                    true
                });
            })
    }
}

struct HooksDemo;

impl Render for HooksDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.tracked(|cx| {
            // Two independent stateful counters — each owns its state via
            // use_signal with a different key. Render them sequentially
            // (each call borrows window/cx mutably and returns an owned
            // element, so there's no overlap).
            let counter_a = StatefulCounter::render_component(_window, cx, "counter-a");
            let counter_b = StatefulCounter::render_component(_window, cx, "counter-b");

            div()
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .size_full()
                .bg(rgb(0x202124))
                .text_color(rgb(0xf4f4f5))
                .child(div().text_lg().child("Component-internal hooks"))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xa1a1aa))
                        .child(
                            "Each counter owns its state via window.use_signal. \
                             No Binding or host callback needed — click to increment.",
                        ),
                )
                .child(counter_a)
                .child(counter_b)
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
            |_, cx| cx.new(|cx| {
                init(cx);
                HooksDemo
            }),
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
