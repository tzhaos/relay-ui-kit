//! Mutation - async write operations with optimistic commit and rollback.
//!
//! `use_mutation(...)` models action-driven async work. This example uses
//! `run_optimistic_with_followup(...)` so the server value updates immediately,
//! rolls back on failure or cancel, and then reconciles to the canonical
//! server response on success.
//!
//! Run with `cargo run -p relay --example mutation`.

#![cfg_attr(target_family = "wasm", no_main)]

use std::time::Duration;

use gpui::{
    App, Bounds, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};
use gpui_platform::application;
use relay::{Binding, Mutation, ReactiveAppExt, ReactiveContextExt, init, use_mutation};

struct MutationDemo {
    draft: Binding<String>,
    server_value: Binding<String>,
    save: Mutation<String, String>,
}

impl MutationDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        let server_value = cx.binding("Relay v2 foundation".to_string());
        let draft = cx.binding(server_value.get(cx));
        let save = use_mutation(cx);

        Self {
            draft,
            server_value,
            save,
        }
    }

    fn cycle_draft(&self, cx: &mut App) {
        let next = match self.draft.signal().get_untracked().as_str() {
            "Relay v2 foundation" => "Source-driven query",
            "Source-driven query" => "error",
            _ => "Relay v2 foundation",
        };
        self.draft.set(cx, next.to_string());
    }

    fn save_current(&self, cx: &mut App) {
        let optimistic_value = self.draft.get(cx);
        let request_value = optimistic_value.clone();
        let server_value_for_optimistic = self.server_value.clone();
        let server_value_for_followup = self.server_value.clone();

        self.save.run_optimistic_with_followup(
            cx,
            move |cx| {
                let previous = server_value_for_optimistic.get(cx);
                server_value_for_optimistic.set(cx, optimistic_value);
                move |cx| {
                    server_value_for_optimistic.set(cx, previous);
                }
            },
            move |cx| async move {
                cx.background_executor()
                    .timer(Duration::from_millis(900))
                    .await;
                if request_value == "error" {
                    Err("Server rejected this payload".to_string())
                } else {
                    Ok(format!("{request_value} (synced)"))
                }
            },
            move |cx, mutation| {
                if let Some(saved) = mutation.last_success_value(cx) {
                    server_value_for_followup.set(cx, saved);
                }
            },
        );
    }
}

impl Render for MutationDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        cx.tracked(|cx| {
            let draft = self.draft.get(cx);
            let server_value = self.server_value.get(cx);
            let (headline, tone, retained) = self.save.fold(
                cx,
                || ("Idle".to_string(), 0xa1a1aa, "No retained success".to_string()),
                |last_success| {
                    (
                        "Saving...".to_string(),
                        0xfbbf24,
                        last_success
                            .cloned()
                            .unwrap_or_else(|| "No retained success".to_string()),
                    )
                },
                |saved| {
                    (
                        format!("Saved successfully: {saved}"),
                        0x4ade80,
                        saved.clone(),
                    )
                },
                |error, last_success| {
                    (
                        format!("Save failed: {error}"),
                        0xef4444,
                        last_success
                            .cloned()
                            .unwrap_or_else(|| "No retained success".to_string()),
                    )
                },
            );

            div()
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .size_full()
                .bg(rgb(0x202124))
                .text_color(rgb(0xf4f4f5))
                .child(div().text_lg().child("Mutation demo"))
                .child(div().text_sm().child(format!("Draft: {draft}")))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x60a5fa))
                        .child(format!("Server value: {server_value}")),
                )
                .child(div().text_sm().text_color(rgb(tone)).child(headline))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0xa1a1aa))
                        .child(format!("Retained last success: {retained}")),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .id("cycle-draft")
                                .px_3()
                                .py_2()
                                .bg(rgb(0x3b82f6))
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x2563eb)))
                                .child("Cycle draft")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.cycle_draft(cx);
                                })),
                        )
                        .child(
                            div()
                                .id("save")
                                .px_3()
                                .py_2()
                                .bg(rgb(0x14b8a6))
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x0f766e)))
                                .child(if self.save.is_pending(cx) {
                                    "Saving..."
                                } else {
                                    "Save draft"
                                })
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.save_current(cx);
                                })),
                        )
                        .child(
                            div()
                                .id("cancel")
                                .px_3()
                                .py_2()
                                .bg(rgb(0xef4444))
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0xdc2626)))
                                .child("Cancel")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.save.cancel(cx);
                                })),
                        ),
                )
                .child(div().text_xs().text_color(rgb(0xa1a1aa)).child(
                    "Saving updates the server value optimistically. Failures and cancel roll back automatically, while success reconciles to the canonical response.",
                ))
        })
    }
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(560.0), px(260.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(MutationDemo::new),
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
