//! Source resource - entity-scoped resource reloads from reactive sources.
//!
//! `StateScope::load_resource_from_source` wires a tracked source snapshot to
//! an initial `Resource::load`, then to `Resource::reload` after source changes.
//! Use it when a GPUI entity owns both the source tracking lifetime and the
//! resource.
//!
//! Run with `cargo run -p relay --example source_resource`.

#![cfg_attr(target_family = "wasm", no_main)]

use std::time::Duration;

use gpui::{
    AnyElement, App, AsyncApp, Bounds, Context, Div, InteractiveElement, IntoElement,
    ParentElement, Render, Stateful, StatefulInteractiveElement, Styled, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_platform::application;
use relay::{
    ReactiveAppExt, ReactiveView, Resource, Signal, StateScope, init, view::reactive_render,
};

struct SourceResourceDemo {
    selected_task: Signal<&'static str>,
    report: Resource<String, String>,
    _scope: StateScope,
}

impl SourceResourceDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        let mut scope = StateScope::new();
        let selected_task = cx.signal("relay/runtime");
        let report = cx.pending_resource::<String, String>();

        let task_for_source = selected_task.clone();
        scope.load_resource_from_source(
            cx,
            report.clone(),
            move |cx| task_for_source.get(cx),
            move |task| move |cx| load_report(cx, task),
        );

        Self {
            selected_task,
            report,
            _scope: scope,
        }
    }

    fn cycle_task(&self, cx: &mut App) {
        let current = self.selected_task.get_untracked();
        let next = match current {
            "relay/runtime" => "relay/uikit",
            "relay/uikit" => "relay/gallery",
            _ => "relay/runtime",
        };
        self.selected_task.set(cx, next);
    }

    fn reload_current(&self, cx: &mut App) {
        let task = self.selected_task.get(cx);
        self.report.reload(cx, move |cx| load_report(cx, task));
    }
}

impl ReactiveView for SourceResourceDemo {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let task = self.selected_task.get(cx);
        let (headline, loading, tone) = self.report.fold_latest(
            cx,
            || ("Preparing report...".to_string(), true, 0xfbbf24),
            |report, loading| {
                (
                    report.clone(),
                    loading,
                    if loading { 0x60a5fa } else { 0x4ade80 },
                )
            },
            |error| (error.clone(), false, 0xef4444),
        );

        div()
            .size_full()
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x202124))
            .text_color(rgb(0xf4f4f5))
            .child(div().text_lg().child("Source resource demo"))
            .child(div().text_sm().child(format!("Selected task: {task}")))
            .child(div().text_sm().text_color(rgb(tone)).child(headline))
            .child(div().text_xs().text_color(rgb(0xa1a1aa)).child(if loading {
                "The previous report remains visible while the selected task reloads."
            } else {
                "Changing the selected task automatically reloads the report."
            }))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        action_button("cycle-task", "Cycle task").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.cycle_task(cx);
                            },
                        )),
                    )
                    .child(
                        action_button("reload-task", "Reload current").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.reload_current(cx);
                            },
                        )),
                    ),
            )
            .into_any_element()
    }
}

impl Render for SourceResourceDemo {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

fn action_button(id: &'static str, label: &'static str) -> Stateful<Div> {
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

fn report_for_task(task: &'static str) -> String {
    format!("Report ready for {task}")
}

async fn load_report(cx: AsyncApp, task: &'static str) -> Result<String, String> {
    cx.background_executor()
        .timer(Duration::from_millis(350))
        .await;
    Ok(report_for_task(task))
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(560.0), px(240.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(SourceResourceDemo::new),
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
    fn source_resource_loads_initially_then_reloads_when_source_changes() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| SourceResourceDemo::new(cx));
        let root = window.root();
        window.draw();

        let initial = app.update_entity(&root, |demo, cx| {
            demo.report.fold_latest(
                cx,
                || ("pending".to_string(), true),
                |report, loading| (report.clone(), loading),
                |error| (error.clone(), false),
            )
        });
        assert_eq!(initial, ("pending".to_string(), true));

        app.update_entity(&root, |demo, cx| {
            demo.report.set_ready(cx, report_for_task("relay/runtime"));
            demo.selected_task.set(cx, "relay/uikit");
        });

        let reloading = app.update_entity(&root, |demo, cx| {
            demo.report.fold_latest(
                cx,
                || ("pending".to_string(), true),
                |report, loading| (report.clone(), loading),
                |error| (error.clone(), false),
            )
        });
        assert_eq!(
            reloading,
            ("Report ready for relay/runtime".to_string(), true)
        );
    }
}
