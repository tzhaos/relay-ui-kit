//! KeyedSubViews — retain one cached GPUI entity per stable item key.
//!
//! Run with `cargo run -p relay --example keyed_subviews`.

#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    AnyElement, App, Bounds, Context, Div, InteractiveElement, IntoElement, KeyDownEvent,
    ParentElement, Render, Stateful, Styled, Window, WindowBounds, WindowOptions, div, prelude::*,
    px, rgb, size,
};
use gpui_platform::application;
use relay::{
    KeyedSubViews, ReactiveAppExt, ReactiveView, Selector, Signal, init, view::reactive_render,
};

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
    selection: Selector<u64>,
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
            selection: cx.selector(Some(1)),
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

    fn toggle_selected(&self, cx: &mut App) {
        let selected = self.selection.get_untracked();
        self.tasks.update(cx, |tasks| {
            let Some(selected) = selected else {
                return false;
            };
            let Some(task) = tasks.iter_mut().find(|task| task.id == selected) else {
                return false;
            };
            task.done = !task.done;
            true
        });
    }

    fn select_next(&self, cx: &mut App) {
        self.tasks.peek(|tasks| {
            self.selection.select_next_by(cx, tasks, |task| task.id);
        });
    }

    fn select_previous(&self, cx: &mut App) {
        self.tasks.peek(|tasks| {
            self.selection.select_previous_by(cx, tasks, |task| task.id);
        });
    }

    fn reverse(&self, cx: &mut App) {
        self.tasks.update(cx, |tasks| {
            tasks.reverse();
            true
        });
    }

    fn handle_key_down(&self, event: &KeyDownEvent, cx: &mut App) -> bool {
        match event.keystroke.key.as_str() {
            "arrow-down" => {
                self.select_next(cx);
                true
            }
            "arrow-up" => {
                self.select_previous(cx);
                true
            }
            "enter" => {
                self.toggle_selected(cx);
                true
            }
            _ => false,
        }
    }
}

impl ReactiveView for TaskList {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let tasks = self.tasks.get(cx);
        self.selection.reconcile_keys_by(cx, &tasks, |task| task.id);

        self.rows.sync(
            cx,
            tasks,
            |task| task.id,
            |task, _cx| TaskRow::new(task, self.selection.clone()),
            |task, row, _cx| row.update_task(task),
        );

        div()
            .id("task-list")
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .size_full()
            .bg(rgb(0x202124))
            .text_color(rgb(0xf4f4f5))
            .tab_index(0)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                if this.handle_key_down(event, cx) {
                    cx.stop_propagation();
                }
            }))
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
                            .child(action_button("previous", "Previous").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.select_previous(cx);
                                },
                            )))
                            .child(action_button("next", "Next").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.select_next(cx);
                                },
                            )))
                            .child(action_button("toggle", "Toggle selected").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.toggle_selected(cx);
                                }),
                            ))
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
    selection: Selector<u64>,
}

impl TaskRow {
    fn new(task: &Task, selection: Selector<u64>) -> Self {
        Self {
            task: task.clone(),
            selection,
        }
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
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let selected = self.selection.is_selected(cx, self.task.id);
        let status = if self.task.done { "done" } else { "open" };
        let tone = if selected {
            0x1e3a8a
        } else if self.task.done {
            0x14532d
        } else {
            0x27272a
        };
        let selection = self.selection.clone();
        let id = self.task.id;

        div()
            .id(format!("task-row-{id}"))
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .py_2()
            .bg(rgb(tone))
            .rounded(px(6.0))
            .cursor_pointer()
            .hover(|style| style.bg(rgb(0x334155)))
            .on_click(move |_, _window, cx| {
                selection.select(cx, id);
                cx.stop_propagation();
            })
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

#[cfg(test)]
mod tests {
    use gpui::{EntityId, Keystroke, TestApp};

    use super::*;

    fn key(name: &str) -> KeyDownEvent {
        KeyDownEvent {
            keystroke: Keystroke {
                key: name.to_string(),
                ..Default::default()
            },
            is_held: false,
            prefer_character_input: false,
        }
    }

    fn row_ids(rows: &KeyedSubViews<u64, TaskRow>) -> Vec<(u64, EntityId)> {
        rows.keyed_iter()
            .map(|(key, view)| (*key, view.entity().entity_id()))
            .collect()
    }

    #[test]
    fn task_list_reuses_rows_when_selection_moves() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| TaskList::new(cx));
        let root = window.root();

        window.draw();
        let initial_rows = app.update_entity(&root, |list, _cx| row_ids(&list.rows));

        app.update_entity(&root, |list, cx| {
            list.select_next(cx);
        });
        window.draw();

        let selected = app.update_entity(&root, |list, _cx| list.selection.get_untracked());
        let updated_rows = app.update_entity(&root, |list, _cx| row_ids(&list.rows));
        assert_eq!(selected, Some(2));
        assert_eq!(updated_rows, initial_rows);
    }

    #[test]
    fn task_list_select_previous_wraps_to_last_task() {
        let mut app = TestApp::new();
        let window = app.open_window(|_, cx| TaskList::new(cx));
        let root = window.root();

        app.update_entity(&root, |list, cx| {
            list.select_previous(cx);
        });

        let selected = app.update_entity(&root, |list, _cx| list.selection.get_untracked());
        assert_eq!(selected, Some(3));
    }

    #[test]
    fn task_list_keyboard_enter_toggles_selected_task() {
        let mut app = TestApp::new();
        let window = app.open_window(|_, cx| TaskList::new(cx));
        let root = window.root();

        app.update_entity(&root, |list, cx| {
            assert!(list.handle_key_down(&key("arrow-down"), cx));
            assert!(list.handle_key_down(&key("enter"), cx));
        });

        let tasks = app.update_entity(&root, |list, _cx| list.tasks.get_untracked());
        assert!(!tasks[0].done);
        assert!(tasks[1].done);
        assert!(!tasks[2].done);
    }

    #[test]
    fn task_list_keyboard_ignores_unhandled_keys() {
        let mut app = TestApp::new();
        let window = app.open_window(|_, cx| TaskList::new(cx));
        let root = window.root();

        let handled = app.update_entity(&root, |list, cx| list.handle_key_down(&key("tab"), cx));

        assert!(!handled);
    }
}
