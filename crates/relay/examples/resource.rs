//! Resource — async state with pending/reloading/ready/error tracking.
//!
//! `Resource<T, E>` wraps a `Signal<ResourceState<T, E>>` and provides `load`
//! to start a reset load on GPUI's foreground executor. `reload` retains the
//! latest value while refreshing. Stale loads are automatically ignored when a
//! newer load is started.
//!
//! Run with `cargo run -p relay --example resource`.

#![cfg_attr(target_family = "wasm", no_main)]

use std::time::Duration;

use gpui::{
    App, Bounds, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};
use gpui_platform::application;
use relay::{ReactiveContextExt, Resource, ResourceState, init};

struct ResourceDemo {
    data: Resource<String, String>,
}

impl ResourceDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        Self {
            data: Resource::pending(cx),
        }
    }

    fn load_success(&self, cx: &mut App) {
        self.data.load(cx, |cx| async move {
            cx.background_executor().timer(Duration::from_secs(2)).await;
            Ok("Data loaded successfully!".into())
        });
    }

    fn reload_success(&self, cx: &mut App) {
        self.data.reload(cx, |cx| async move {
            cx.background_executor().timer(Duration::from_secs(2)).await;
            Ok("Data refreshed successfully!".into())
        });
    }

    fn load_failure(&self, cx: &mut App) {
        self.data.load(cx, |cx| async move {
            cx.background_executor().timer(Duration::from_secs(1)).await;
            Err("Simulated load failure".into())
        });
    }
}

impl Render for ResourceDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.tracked(|cx| {
            // Clone the state out so we own the strings (avoids lifetime issues
            // with borrowing into 'static closures below).
            let state = self.data.get(cx);
            let latest = self.data.latest(cx);
            let (status_text, status_color) = match &state {
                ResourceState::Pending => ("Loading...".to_string(), rgb(0xfbbf24)),
                ResourceState::Reloading(value) => {
                    (format!("{value} (refreshing...)"), rgb(0x60a5fa))
                }
                ResourceState::Ready(v) => (v.clone(), rgb(0x4ade80)),
                ResourceState::Error(e) => (e.clone(), rgb(0xef4444)),
            };
            let latest_text = latest.unwrap_or_else(|| "No latest value".to_string());

            div()
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .size_full()
                .bg(rgb(0x202124))
                .text_color(rgb(0xf4f4f5))
                .child(div().text_lg().child("Resource demo"))
                .child(div().text_sm().text_color(status_color).child(status_text))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0xa1a1aa))
                        .child(format!("latest: {latest_text}")),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .id("load")
                                .px_3()
                                .py_2()
                                .bg(rgb(0x3b82f6))
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x2563eb)))
                                .child("Load (2s delay)")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.load_success(cx);
                                })),
                        )
                        .child(
                            div()
                                .id("reload")
                                .px_3()
                                .py_2()
                                .bg(rgb(0x14b8a6))
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x0f766e)))
                                .child("Reload (keeps latest)")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.reload_success(cx);
                                })),
                        )
                        .child(
                            div()
                                .id("load-error")
                                .px_3()
                                .py_2()
                                .bg(rgb(0xef4444))
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0xdc2626)))
                                .child("Load error (1s delay)")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.load_failure(cx);
                                })),
                        ),
                )
                .child(div().text_xs().text_color(rgb(0xa1a1aa)).child(
                    "load resets to pending; reload keeps the previous ready value visible.",
                ))
        })
    }
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(520.0), px(240.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(ResourceDemo::new),
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
