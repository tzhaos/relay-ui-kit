use gpui::{Context, IntoElement, ParentElement, Styled, div, px};
use relay_uikit::patterns::{
    OutputLog, OutputSurface, Pane, PaneToolbar, TabStrip, TopToolbar, WorkspaceBreadcrumb,
    output_resource_snapshot,
};
use relay_uikit::{IconButton, IconName, Theme};

use super::{WorkbenchApp, WorkbenchState};

pub(super) fn center_pane(
    state: &WorkbenchState,
    theme: Theme,
    cx: &mut Context<WorkbenchApp>,
) -> impl IntoElement + use<> {
    let task = state.selected_task.get(cx);
    let session = state.selected_session.get(cx);
    let connected = session.as_ref().is_some_and(|session| session.connected);
    let output = output_resource_snapshot(
        &state.terminal_output,
        cx,
        "Loading terminal output",
        "Refreshing terminal output",
        |line_count| format!("{line_count} lines ready"),
        "Terminal refresh failed",
        |error| format!("terminal refresh failed: {error}"),
    );
    let output_loading = output.loading;
    let output_status = output.status_text;
    let output_lines = output.lines;
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
                    OutputSurface::new(
                        "workbench-output",
                        OutputLog::new(output_lines).prompt("> "),
                    )
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
                    .center(
                        div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child(output_status),
                    )
                    .trailing(
                        PaneToolbar::new()
                            .action(
                                IconButton::new("workbench-refresh", IconName::RefreshCw)
                                    .aria_label("Refresh terminal output")
                                    .active(output_loading)
                                    .disabled(output_loading)
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.state.refresh_terminal_output(cx);
                                    })),
                            )
                            .action(
                                IconButton::new("workbench-more", IconName::Ellipsis)
                                    .aria_label("Open terminal actions"),
                            ),
                    ),
            ),
    )
}

fn session_tabs(
    state: &WorkbenchState,
    sessions: &[super::data::WorkbenchSession],
) -> impl IntoElement + use<> {
    let selection = state.active_session.selection().selector().clone();

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
            .active_by(selection.clone(), session.id)
        }))
}
