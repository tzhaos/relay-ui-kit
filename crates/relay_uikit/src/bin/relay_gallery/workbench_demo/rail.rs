use gpui::{App, FontWeight, IntoElement, ParentElement, Styled, div, px};
use relay_uikit::patterns::{Pane, PaneSurface, PaneWidth};
use relay_uikit::workbench::{TaskRow, TaskRowData, TerminalStatusBadge};
use relay_uikit::{
    Divider, IconButton, IconName, NavRow, PanelHeader, Theme, TreeRow,
    theme::{radius, space},
};

use super::{
    WorkbenchState,
    data::{DEMO_TASKS, active_session, session_index_for_key},
};

pub(super) fn left_rail(
    state: &WorkbenchState,
    theme: Theme,
    cx: &App,
) -> impl IntoElement {
    let body = div()
        .size_full()
        .flex()
        .flex_col()
        .child(nav_section(state, theme, cx))
        .child(Divider::horizontal())
        .child(project_tree(theme))
        .child(Divider::horizontal())
        .child(tasks_header(state, theme))
        .child(task_rows(state, cx));

    Pane::new(PaneWidth::Flex, body)
        .surface(PaneSurface::Chrome)
        .header(PanelHeader::new("Workspace").icon(IconName::Folder))
}

fn nav_section(
    state: &WorkbenchState,
    theme: Theme,
    cx: &App,
) -> impl IntoElement {
    let route = state.route.clone();
    let context_tab = state.context_tab.clone();

    div()
        .px_2()
        .py_2()
        .flex()
        .flex_col()
        .gap(px(1.0))
        .child(
            NavRow::new("nav-tasks", IconName::ListChecks, "Tasks")
                .selected(true)
                .count(DEMO_TASKS.len()),
        )
        .child(
            NavRow::new("nav-terminals", IconName::Terminal, "Terminals").on_click({
                let route = route.clone();
                move |_event, _window, cx| {
                    route.set(cx, "terminal");
                }
            }),
        )
        .child(
            NavRow::new("nav-search", IconName::Search, "Search").on_click({
                let context_tab = context_tab.clone();
                move |_event, _window, cx| {
                    context_tab.set(cx, "files");
                }
            }),
        )
        .child(session_summary(state, theme, cx))
}

fn session_summary(
    state: &WorkbenchState,
    theme: Theme,
    cx: &App,
) -> impl IntoElement {
    let session = active_session(state.active_session.get(cx));

    div()
        .mt_1()
        .px_2()
        .py_1()
        .rounded(px(radius::MD))
        .bg(gpui::transparent_black())
        .flex()
        .items_center()
        .justify_between()
        .gap_2()
        .child(
            div()
                .min_w_0()
                .truncate()
                .text_size(px(11.0))
                .text_color(theme.text_muted)
                .child(format!("Active: {}", session.label)),
        )
        .child(TerminalStatusBadge::new(session.tone))
}

fn project_tree(theme: Theme) -> impl IntoElement {
    div()
        .px_2()
        .py_2()
        .flex()
        .flex_col()
        .gap(px(1.0))
        .child(section_label("PROJECT", theme))
        .child(TreeRow::new("proj-relay", IconName::Folder, "Relay").expandable(true))
        .child(
            TreeRow::new("wt-main", IconName::GitBranch, "main")
                .depth(1)
                .selected(true),
        )
        .child(TreeRow::new("wt-shell", IconName::GitBranch, "ui/workbench-shell").depth(1))
        .child(TreeRow::new("wt-agent", IconName::GitBranch, "fix/agent-retry").depth(1))
}

fn tasks_header(state: &WorkbenchState, theme: Theme) -> impl IntoElement {
    let active_task = state.active_task.clone();
    let active_session = state.active_session.clone();

    div()
        .px_3()
        .h(px(space::PANE_HEADER))
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_size(px(11.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text_muted)
                .child("TASKS"),
        )
        .child(IconButton::new("add-task", IconName::Plus).on_click({
            move |_event, _window, cx| {
                active_task.set(cx, 0);
                active_session.set(cx, session_index_for_key(DEMO_TASKS[0].session_key));
            }
        }))
}

fn task_rows(
    state: &WorkbenchState,
    cx: &App,
) -> impl IntoElement {
    let active_task = state.active_task.clone();
    let active_session = state.active_session.clone();
    let route = state.route.clone();

    div()
        .flex_1()
        .min_h_0()
        .px_2()
        .pb_2()
        .flex()
        .flex_col()
        .gap_1()
        .children(DEMO_TASKS.iter().enumerate().map(|(index, task)| {
            let active_task = active_task.clone();
            let active_session = active_session.clone();
            let route = route.clone();
            TaskRow::new(
                ("task", index),
                TaskRowData {
                    title: task.title.into(),
                    status_label: task.status.into(),
                    status_tone: task.tone,
                    branch: Some(task.branch.into()),
                    changed: task.changed,
                    review: task.review,
                },
            )
            .selected(index == state.active_task.get(cx))
            .on_click(move |_event, _window, cx| {
                active_task.set(cx, index);
                active_session.set(cx, session_index_for_key(task.session_key));
                route.set(cx, "terminal");
            })
            .into_any_element()
        }))
}

fn section_label(label: &'static str, theme: Theme) -> impl IntoElement {
    div()
        .text_size(px(11.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme.text_muted)
        .child(label)
}
