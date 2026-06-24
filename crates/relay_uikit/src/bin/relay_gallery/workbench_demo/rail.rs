use gpui::{
    AnyElement, AnyView, App, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
    div, px,
};
use relay::{
    KeyedSubViews, OrderedSelectionModel, ReactiveView, Selector, Signal, SignalVecExt,
    view::reactive_render,
};
use relay_uikit::patterns::{Pane, PaneToolbar, TaskRow};
use relay_uikit::{Button, IconButton, IconName, TextInput, Theme};

use super::{WorkbenchApp, WorkbenchState, data::WorkbenchTask};

pub(super) fn left_rail(
    state: &WorkbenchState,
    window: &Window,
    theme: Theme,
    _cx: &mut Context<WorkbenchApp>,
) -> impl IntoElement + use<> {
    let task_list = state.task_list.clone();
    let next_task = state.task_list.clone();
    let previous_task = state.task_list.clone();
    let remove_task = state.task_list.clone();
    let focused = state.filter_focus.is_focused(window);

    Pane::rail(
        div()
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .child(
                TextInput::bound(
                    "workbench-task-filter",
                    state.filter_focus.clone(),
                    state.filter.clone(),
                )
                .placeholder("Filter tasks")
                .leading_icon(IconName::Search)
                .focused(focused),
            )
            .child(cached_task_list(task_list)),
    )
    .header(
        div()
            .h(px(40.0))
            .px_3()
            .flex()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .text_color(theme.text_secondary)
                    .child("Tasks"),
            )
            .child(
                PaneToolbar::new()
                    .action(
                        IconButton::new("workbench-task-prev", IconName::ArrowLeft)
                            .aria_label("Previous task")
                            .on_click(move |_event, _window, cx| {
                                previous_task.update(cx, |list, cx| {
                                    list.activate_previous(cx);
                                });
                            }),
                    )
                    .action(
                        IconButton::new("workbench-task-next", IconName::ArrowRight)
                            .aria_label("Next task")
                            .on_click(move |_event, _window, cx| {
                                next_task.update(cx, |list, cx| {
                                    list.activate_next(cx);
                                });
                            }),
                    )
                    .action(
                        Button::new("workbench-task-remove", "Remove")
                            .ghost()
                            .icon(IconName::Trash2)
                            .on_click(move |_event, _window, cx| {
                                remove_task.update(cx, |list, cx| {
                                    list.remove_active(cx);
                                });
                            }),
                    ),
            ),
    )
}

fn cached_task_list(list: Entity<TaskListView>) -> AnyElement {
    let view: AnyView = list.into();
    view.cached(gpui::StyleRefinement::default().w_full())
        .into_any_element()
}

pub(super) struct TaskListView {
    tasks: Signal<Vec<WorkbenchTask>>,
    rows: KeyedSubViews<u64, TaskRowView>,
    selection: OrderedSelectionModel<u64>,
}

impl TaskListView {
    pub(super) fn new(
        cx: &mut Context<Self>,
        tasks: Signal<Vec<WorkbenchTask>>,
        selection: OrderedSelectionModel<u64>,
    ) -> Self {
        relay::init(cx);
        Self {
            tasks,
            rows: KeyedSubViews::new(),
            selection,
        }
    }

    fn activate_next(&self, cx: &mut App) {
        let _ = self.selection.select_next(cx);
    }

    fn activate_previous(&self, cx: &mut App) {
        let _ = self.selection.select_previous(cx);
    }

    fn remove_active(&self, cx: &mut App) {
        self.tasks
            .remove_selected_by(cx, self.selection.selection().selector(), |task| task.id);
    }
}

impl ReactiveView for TaskListView {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let tasks = self.tasks.get(cx);
        let selector = self.selection.selection().selector().clone();
        self.rows.sync_with_selector(
            cx,
            self.selection.selection().selector(),
            tasks,
            |task| task.id,
            move |task, _cx| TaskRowView::new(task, selector.clone()),
            |task, row, _cx| row.update_task(task),
        );

        div()
            .w_full()
            .min_h_0()
            .flex()
            .flex_col()
            .gap_1()
            .children(self.rows.cached(gpui::StyleRefinement::default().w_full()))
            .into_any_element()
    }
}

impl Render for TaskListView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

struct TaskRowView {
    task: WorkbenchTask,
    selection: Selector<u64>,
}

impl TaskRowView {
    fn new(task: &WorkbenchTask, selection: Selector<u64>) -> Self {
        Self {
            task: task.clone(),
            selection,
        }
    }

    fn update_task(&mut self, task: &WorkbenchTask) -> bool {
        if self.task == *task {
            false
        } else {
            self.task = task.clone();
            true
        }
    }
}

impl ReactiveView for TaskRowView {
    fn render_state(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> AnyElement {
        TaskRow::new(
            format!("workbench-task-{}", self.task.id),
            self.task.row_data(),
        )
        .selected_by(self.selection.clone(), self.task.id)
        .into_any_element()
    }
}

impl Render for TaskRowView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

#[cfg(test)]
mod tests {
    use gpui::{EntityId, TestApp};
    use relay::ReactiveAppExt;

    use super::*;
    use crate::workbench_demo::data::initial_tasks;

    fn row_ids(rows: &KeyedSubViews<u64, TaskRowView>) -> Vec<(u64, EntityId)> {
        rows.keyed_iter()
            .map(|(key, view)| (*key, view.entity().entity_id()))
            .collect()
    }

    #[test]
    fn task_list_reuses_rows_when_selection_changes() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| {
            relay_uikit::theme::init(cx);
            let tasks = cx.signal(initial_tasks());
            let tasks_for_selection = tasks.clone();
            let selection = relay::use_ordered_selection_model(
                cx,
                Some(1),
                move |cx| {
                    tasks_for_selection.read(cx, |tasks| tasks.iter().map(|task| task.id).collect())
                },
                relay::SelectionReconcilePolicy::SelectFirst,
            );
            TaskListView::new(cx, tasks, selection)
        });
        let root = window.root();

        window.draw();
        let initial_rows = app.update_entity(&root, |list, _cx| row_ids(&list.rows));

        app.update_entity(&root, |list, cx| {
            list.activate_next(cx);
        });
        window.draw();

        let selected = app.update_entity(&root, |list, _cx| list.selection.get_untracked());
        let updated_rows = app.update_entity(&root, |list, _cx| row_ids(&list.rows));
        assert_eq!(selected, Some(2));
        assert_eq!(updated_rows, initial_rows);
    }

    #[test]
    fn task_list_remove_active_reselects_first_remaining_task() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| {
            relay_uikit::theme::init(cx);
            let tasks = cx.signal(initial_tasks());
            let tasks_for_selection = tasks.clone();
            let selection = relay::use_ordered_selection_model(
                cx,
                Some(1),
                move |cx| {
                    tasks_for_selection.read(cx, |tasks| tasks.iter().map(|task| task.id).collect())
                },
                relay::SelectionReconcilePolicy::SelectFirst,
            );
            TaskListView::new(cx, tasks, selection)
        });
        let root = window.root();

        window.draw();
        app.update_entity(&root, |list, cx| {
            list.activate_next(cx);
            list.remove_active(cx);
        });
        window.draw();

        let (selected, tasks) = app.update_entity(&root, |list, _cx| {
            (
                list.selection.get_untracked(),
                list.tasks
                    .get_untracked()
                    .into_iter()
                    .map(|task| task.id)
                    .collect::<Vec<_>>(),
            )
        });
        assert_eq!(selected, Some(1));
        assert_eq!(tasks, vec![1, 3]);
    }
}
