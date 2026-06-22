use gpui::{div, px, Context, IntoElement, ParentElement, Styled};
use relay_uikit::patterns::{
    OutputLog, OutputSurface, Pane, PaneToolbar, TabStrip, TopToolbar, WorkspaceBreadcrumb,
};
use relay_uikit::{IconButton, IconName, Theme};

use super::{data::terminal_lines, WorkbenchApp, WorkbenchState};

pub(super) fn center_pane(
    state: &WorkbenchState,
    theme: Theme,
    cx: &mut Context<WorkbenchApp>,
) -> impl IntoElement + use<> {
    let task = state.selected_task.get(cx);
    let session = state.selected_session.get(cx);
    let connected = session.as_ref().is_some_and(|session| session.connected);
    let lines = terminal_lines(task.as_ref(), session.as_ref());
    let active_task_title = task.as_ref().map_or("No task", |task| task.title.as_str());
    let tabs = state
        .sessions
        .read(cx, |sessions| session_tabs(state, sessions));

    Pane::center(
        div()
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .child(tabs)
            .child(
                div().flex_1().min_h_0().child(
                    OutputSurface::new("workbench-output", OutputLog::new(lines).prompt("> "))
                        .connected(connected),
                ),
            ),
    )
    .header(
        div()
            .h(px(44.0))
            .px_3()
            .border_b_1()
            .border_color(theme.border)
            .child(
                TopToolbar::new()
                    .leading(WorkspaceBreadcrumb::new(vec![
                        "relay-ui-kit".into(),
                        "workbench".into(),
                        active_task_title.into(),
                    ]))
                    .trailing(
                        PaneToolbar::new()
                            .action(IconButton::new("workbench-refresh", IconName::RefreshCw))
                            .action(IconButton::new("workbench-more", IconName::Ellipsis)),
                    ),
            ),
    )
}

fn session_tabs(
    state: &WorkbenchState,
    sessions: &[super::data::WorkbenchSession],
) -> impl IntoElement + use<> {
    div()
        .h(px(40.0))
        .px_2()
        .flex()
        .items_center()
        .gap_1()
        .border_b_1()
        .border_color(gpui::transparent_black())
        .children(sessions.iter().map(|session| {
            TabStrip::new(
                format!("workbench-session-tab-{}", session.id),
                session.label.clone(),
            )
            .status(session.tone)
            .active_by(state.active_session.clone(), session.id)
        }))
}
