//! KeyedSubViews — retain one cached GPUI entity per stable item key.
//!
//! Run with `cargo run -p relay --example keyed_subviews`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    AnyElement, App, Bounds, Context, Div, InteractiveElement, IntoElement, ParentElement, Render,
    Stateful, Styled, Window, WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_platform::application;
use relay::{KeyedSubViews, ReactiveAppExt, ReactiveView, Signal, init, view::reactive_render};

#[derive(Clone, PartialEq, Eq)]
struct Task {
    id: u64,
    title: String,
    done: bool,
}

impl Task {
    fn new(id: u64, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            done: false,
        }
    }
}

struct TaskList {
    tasks: Signal<Vec<Task>>,
    rows: KeyedSubViews<u64, TaskRow>,
    next_id: u64,
}

impl TaskList {
    fn new(cx: &mut Context<Self>) -> Self {
        init(cx);
        Self {
            tasks: cx.signal(vec![
                Task::new(1, "Design panel"),
                Task::new(2, "Wire state"),
                Task::new(3, "Review cache"),
            ]),
            rows: KeyedSubViews::new(),
            next_id: 4,
        }
    }

    fn add_task(&mut self, cx: &mut App) {
        let id = self.next_id;
        self.next_id += 1;
        self.tasks.update(cx, |tasks| {
            tasks.push(Task::new(id, format!("Task {id}")));
            true
        });
    }

    fn toggle_first(&self, cx: &mut App) {
        self.tasks.update(cx, |tasks| {
            if let Some(task) = tasks.first_mut() {
                task.done = !task.done;
                true
            } else {
                false
            }
        });
    }

    fn reverse(&self, cx: &mut App) {
        self.tasks.update(cx, |tasks| {
            tasks.reverse();
            true
        });
    }
}

impl ReactiveView for TaskList {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let tasks = self.tasks.get(cx);

        self.rows.sync(
            cx,
            tasks,
            |task| task.id,
            |task, _cx| TaskRow::new(task),
            |task, row, _cx| row.update_task(task),
        );

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
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(div().text_lg().child("KeyedSubViews"))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                action_button("toggle", "Toggle first").on_click(cx.listener(
                                    |this, _, _, cx| {
                                        this.toggle_first(cx);
                                    },
                                )),
                            )
                            .child(action_button("reverse", "Reverse").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.reverse(cx);
                                },
                            )))
                            .child(action_button("add", "Add").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.add_task(cx);
                                },
                            ))),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .children(self.rows.cached(gpui::StyleRefinement::default().w_full())),
            )
            .into_any_element()
    }
}

impl Render for TaskList {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

struct TaskRow {
    task: Task,
}

impl TaskRow {
    fn new(task: &Task) -> Self {
        Self { task: task.clone() }
    }

    fn update_task(&mut self, task: &Task) -> bool {
        if self.task == *task {
            false
        } else {
            self.task = task.clone();
            true
        }
    }
}

impl ReactiveView for TaskRow {
    fn render_state(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> AnyElement {
        let status = if self.task.done { "done" } else { "open" };
        let tone = if self.task.done { 0x14532d } else { 0x27272a };

        div()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .py_2()
            .bg(rgb(tone))
            .rounded(px(6.0))
            .child(div().child(self.task.title.clone()))
            .child(div().text_xs().text_color(rgb(0xa1a1aa)).child(status))
            .into_any_element()
    }
}

impl Render for TaskRow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

fn action_button(id: &'static str, label: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .px_2()
        .py_1()
        .bg(rgb(0x3b82f6))
        .rounded(px(4.0))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(0x2563eb)))
        .text_xs()
        .child(label)
}

fn run_example() {
    application().run(|cx: &mut App| {
        init(cx);
        let bounds = Bounds::centered(None, size(px(560.0), px(320.0)), cx);
        let _ = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(TaskList::new),
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
