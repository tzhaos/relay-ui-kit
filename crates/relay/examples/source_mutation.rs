//! Source query + mutation - refresh the latest source after a write.
//!
//! This example shows the intended relay_v2 async flow:
//! - a [`SourceQuery`] owns read-side async state tied to reactive source data
//! - a [`Mutation`] owns action-driven write-side async state
//! - mutation follow-up calls [`SourceQuery::invalidate`] to refetch from the
//!   latest source without manually rebuilding the read query in the view
//!
//! Run with `cargo run -p relay --example source_mutation`.

#![cfg_attr(target_family = "wasm", no_main)]

use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Duration};

use gpui::{
    AnyElement, App, AsyncApp, Bounds, Context, Div, InteractiveElement, IntoElement,
    ParentElement, Render, Stateful, StatefulInteractiveElement, Styled, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_platform::application;
use relay::{
    Mutation, ReactiveAppExt, ReactiveView, Signal, SourceQuery, init, use_mutation,
    use_query_from_source, view::reactive_render,
};

type ReportVersions = Rc<RefCell<HashMap<&'static str, usize>>>;

struct SourceMutationDemo {
    selected_task: Signal<&'static str>,
    report: SourceQuery<String, String>,
    publish: Mutation<usize, String>,
    report_versions: ReportVersions,
}

impl SourceMutationDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        let selected_task = cx.signal("relay/runtime");
        let report_versions = Rc::new(RefCell::new(HashMap::from([
            ("relay/runtime", 1),
            ("relay/uikit", 2),
            ("relay/gallery", 3),
        ])));

        let task_for_query = selected_task.clone();
        let versions_for_query = report_versions.clone();
        let report = use_query_from_source(
            cx,
            move |cx| task_for_query.get(cx),
            move |task| {
                let versions_for_query = versions_for_query.clone();
                move |cx| load_report(cx, versions_for_query, task)
            },
        );
        let publish = use_mutation(cx);

        Self {
            selected_task,
            report,
            publish,
            report_versions,
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

    fn refresh_report(&self, cx: &mut App) {
        self.report.reload(cx);
    }

    fn publish_current(&self, cx: &mut App) {
        let task = self.selected_task.get(cx);
        let report_versions = self.report_versions.clone();
        let report = self.report.clone();

        self.publish.run_with_followup(
            cx,
            move |cx| publish_report(cx, report_versions, task),
            move |cx, _mutation| {
                report.invalidate(cx);
            },
        );
    }
}

impl ReactiveView for SourceMutationDemo {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let task = self.selected_task.get(cx);
        let (headline, loading, tone) = self.report.query().fold_latest(
            cx,
            || ("Loading server report...".to_string(), true, 0xfbbf24),
            |report, loading| {
                (
                    report.clone(),
                    loading,
                    if loading { 0x60a5fa } else { 0x4ade80 },
                )
            },
            |error| (error.clone(), false, 0xef4444),
        );
        let publish_status = self.publish.fold(
            cx,
            || "No publish yet".to_string(),
            |_| "Publishing update...".to_string(),
            |version| format!("Published current task. Server version is now v{version}."),
            |error, last_success| match last_success {
                Some(version) => {
                    format!("Publish failed: {error}. Last successful server version: v{version}.")
                }
                None => format!("Publish failed: {error}."),
            },
        );

        div()
            .size_full()
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x202124))
            .text_color(rgb(0xf4f4f5))
            .child(div().text_lg().child("Source query + mutation demo"))
            .child(div().text_sm().child(format!("Selected task: {task}")))
            .child(div().text_sm().text_color(rgb(tone)).child(headline))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x14b8a6))
                    .child(publish_status),
            )
            .child(div().text_xs().text_color(rgb(0xa1a1aa)).child(if loading {
                "The last report stays visible while the current task refreshes."
            } else {
                "Publishing does not change the selected task. It invalidates the source query and refetches the latest server report."
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
                        action_button("refresh-report", "Refresh report").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.refresh_report(cx);
                            },
                        )),
                    )
                    .child(
                        action_button("publish-task", "Publish update")
                            .bg(rgb(0x14b8a6))
                            .hover(|style| style.bg(rgb(0x0f766e)))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.publish_current(cx);
                            })),
                    ),
            )
            .into_any_element()
    }
}

impl Render for SourceMutationDemo {
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

async fn load_report(
    cx: AsyncApp,
    report_versions: ReportVersions,
    task: &'static str,
) -> Result<String, String> {
    cx.background_executor()
        .timer(Duration::from_millis(350))
        .await;

    let version = report_versions.borrow().get(task).copied().unwrap_or(1);
    Ok(format!("Report ready for {task} (server v{version})"))
}

async fn publish_report(
    cx: AsyncApp,
    report_versions: ReportVersions,
    task: &'static str,
) -> Result<usize, String> {
    cx.background_executor()
        .timer(Duration::from_millis(600))
        .await;

    let mut report_versions = report_versions.borrow_mut();
    let version = report_versions.entry(task).or_insert(1);
    *version += 1;
    Ok(*version)
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(620.0), px(280.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(SourceMutationDemo::new),
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
