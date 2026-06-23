//! Orca-direction workbench sample assembled from the Relay UI crates.

use std::{fmt, time::Duration};

mod center;
mod context;
mod data;
mod rail;

use gpui::App;
use gpui::{
    AnyElement, AppContext, AsyncApp, Context, Entity, FocusHandle, IntoElement, Render, Window,
};
use relay::{
    Binding, Memo, OrderedSelectionModel, ReactiveAppExt, Resource, SelectionReconcilePolicy,
    Signal, use_ordered_selection_model,
    view::{ReactiveView, StateScope, reactive_render},
};
use relay_uikit::patterns::{
    AppShell, OutputLine, SplitPane, SplitPaneState, StatusBar, StatusItem,
};
use relay_uikit::{ActiveTheme, TextInputState, Tone, icon::IconName, theme::space};

use center::center_pane;
use context::right_context;
use data::{
    WorkbenchReviewReport, WorkbenchSession, WorkbenchTask, initial_review_report,
    initial_sessions, initial_tasks, review_report_for_task,
    selected_session as resolve_selected_session, selected_task as resolve_selected_task,
    terminal_lines,
};
use rail::left_rail;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkbenchContextTab {
    Files,
    Sessions,
    Review,
}

impl WorkbenchContextTab {
    pub fn label(self) -> &'static str {
        match self {
            Self::Files => "Files",
            Self::Sessions => "Sessions",
            Self::Review => "Review",
        }
    }
}

impl fmt::Display for WorkbenchContextTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

pub struct WorkbenchApp {
    pub state: WorkbenchState,
    pub _scope: StateScope,
}

impl WorkbenchApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut scope = StateScope::new();
        let state = WorkbenchState::new(cx);
        state.watch_terminal_output_sources(cx, &mut scope);
        state.watch_review_report_sources(cx, &mut scope);
        Self {
            state,
            _scope: scope,
        }
    }
}

/// Interactive state for the Workbench page.
pub struct WorkbenchState {
    tasks: Signal<Vec<WorkbenchTask>>,
    sessions: Signal<Vec<WorkbenchSession>>,
    active_task: OrderedSelectionModel<u64>,
    active_session: OrderedSelectionModel<u64>,
    selected_task: Memo<Option<WorkbenchTask>>,
    selected_session: Memo<Option<WorkbenchSession>>,
    terminal_output: Resource<Vec<OutputLine>, String>,
    review_report: Resource<WorkbenchReviewReport, String>,
    task_list: Entity<rail::TaskListView>,
    session_list: Entity<context::SessionListView>,
    pub context_tab: Binding<WorkbenchContextTab>,
    pub filter: Binding<TextInputState>,
    pub filter_focus: FocusHandle,
    pub left_split: Entity<SplitPaneState>,
    pub terminal_split: Entity<SplitPaneState>,
}

impl WorkbenchState {
    pub fn new(cx: &mut Context<WorkbenchApp>) -> Self {
        let task_seed = initial_tasks();
        let session_seed = initial_sessions();
        let initial_terminal_output = terminal_lines(
            resolve_selected_task(&task_seed, Some(1)),
            resolve_selected_session(&session_seed, Some(11)),
        );
        let tasks = cx.signal(task_seed);
        let sessions = cx.signal(session_seed);
        let tasks_for_selection = tasks.clone();
        let active_task = use_ordered_selection_model(
            cx,
            Some(1),
            move |cx| {
                tasks_for_selection.read(cx, |tasks| tasks.iter().map(|task| task.id).collect())
            },
            SelectionReconcilePolicy::SelectFirst,
        );
        let sessions_for_selection = sessions.clone();
        let active_session = use_ordered_selection_model(
            cx,
            Some(11),
            move |cx| {
                sessions_for_selection.read(cx, |sessions| {
                    sessions.iter().map(|session| session.id).collect()
                })
            },
            SelectionReconcilePolicy::SelectFirst,
        );
        let terminal_output = cx.ready_resource(initial_terminal_output);
        let review_report = cx.ready_resource(initial_review_report());
        let selected_task = active_task.selected_from_signal(cx, &tasks, |task| task.id);
        let selected_session =
            active_session.selected_from_signal(cx, &sessions, |session| session.id);
        let task_list = cx.new({
            let tasks = tasks.clone();
            let active_task = active_task.clone();
            move |cx| rail::TaskListView::new(cx, tasks, active_task)
        });
        let session_list = cx.new({
            let sessions = sessions.clone();
            let active_session = active_session.clone();
            move |cx| context::SessionListView::new(cx, sessions, active_session)
        });

        Self {
            tasks,
            sessions,
            active_task,
            active_session,
            selected_task,
            selected_session,
            terminal_output,
            review_report,
            task_list,
            session_list,
            context_tab: cx.binding(WorkbenchContextTab::Files),
            filter: cx.binding(TextInputState::new()),
            filter_focus: cx.focus_handle(),
            left_split: cx.new(|_| SplitPaneState::new(space::RAIL_WIDTH)),
            terminal_split: cx.new(|_| SplitPaneState::new(760.0)),
        }
    }

    fn refresh_review_report(&self, cx: &mut App) {
        let task = self.selected_task.get(cx);
        reload_review_report(self.review_report.clone(), task, cx);
    }

    fn refresh_terminal_output(&self, cx: &mut App) {
        let task = self.selected_task.get(cx);
        let session = self.selected_session.get(cx);
        reload_terminal_output(self.terminal_output.clone(), task, session, cx);
    }

    fn watch_terminal_output_sources(
        &self,
        cx: &mut Context<WorkbenchApp>,
        scope: &mut StateScope,
    ) {
        let selected_task = self.selected_task.clone();
        let selected_session = self.selected_session.clone();
        let terminal_output = self.terminal_output.clone();

        scope.reload_resource_from_source(
            cx,
            terminal_output,
            move |cx| (selected_task.get(cx), selected_session.get(cx)),
            move |(task, session)| move |cx| load_terminal_output(cx, task, session),
        );
    }

    fn watch_review_report_sources(&self, cx: &mut Context<WorkbenchApp>, scope: &mut StateScope) {
        let selected_task = self.selected_task.clone();
        let review_report = self.review_report.clone();

        scope.reload_resource_from_source(
            cx,
            review_report,
            move |cx| selected_task.get(cx),
            move |task| move |cx| load_review_report(cx, task),
        );
    }
}

fn reload_terminal_output(
    terminal_output: Resource<Vec<OutputLine>, String>,
    task: Option<WorkbenchTask>,
    session: Option<WorkbenchSession>,
    cx: &mut App,
) {
    terminal_output.reload(cx, move |cx| load_terminal_output(cx, task, session));
}

async fn load_terminal_output(
    cx: AsyncApp,
    task: Option<WorkbenchTask>,
    session: Option<WorkbenchSession>,
) -> Result<Vec<OutputLine>, String> {
    cx.background_executor()
        .timer(Duration::from_millis(500))
        .await;
    Ok(terminal_lines(task.as_ref(), session.as_ref()))
}

fn reload_review_report(
    review_report: Resource<WorkbenchReviewReport, String>,
    task: Option<WorkbenchTask>,
    cx: &mut App,
) {
    review_report.reload(cx, move |cx| load_review_report(cx, task));
}

async fn load_review_report(
    cx: AsyncApp,
    task: Option<WorkbenchTask>,
) -> Result<WorkbenchReviewReport, String> {
    cx.background_executor()
        .timer(Duration::from_millis(650))
        .await;
    Ok(review_report_for_task(task.as_ref()))
}

impl ReactiveView for WorkbenchApp {
    fn render_state(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = *cx.theme();
        let state = &self.state;
        let left = left_rail(state, window, theme, cx);
        let center = center_pane(state, theme, cx);
        let right = right_context(state, window, theme, cx);
        let center_and_context = SplitPane::new("center-context-split", center, right)
            .state(state.terminal_split.clone())
            .min_sizes(560.0, 320.0)
            .first_size(760.0);

        let workbench = SplitPane::new("workbench-left-split", left, center_and_context)
            .state(state.left_split.clone())
            .min_sizes(260.0, 780.0)
            .first_size(space::RAIL_WIDTH);

        AppShell::new(workbench)
            .status_bar(status_bar(state, cx))
            .into_any_element()
    }
}

impl Render for WorkbenchApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

fn status_bar(state: &WorkbenchState, cx: &App) -> impl IntoElement {
    let task = state.selected_task.get(cx);
    let session = state.selected_session.get(cx);
    let worktree = task.as_ref().map_or("No task", |task| task.worktree);
    let task_tone = task.as_ref().map_or(Tone::Muted, |task| task.tone);
    let session_label = session
        .as_ref()
        .map_or("No session", |session| session.label.as_str());
    let session_tone = session.as_ref().map_or(Tone::Muted, |session| session.tone);
    let changed = task.as_ref().map_or(0, |task| task.changed);
    let review = task.as_ref().map_or(0, |task| task.review);
    let task_count = state.tasks.read(cx, |tasks| tasks.len());
    let task_position = state
        .active_task
        .get(cx)
        .map_or_else(|| "-".to_string(), |id| id.to_string());

    StatusBar::new()
        .left(
            StatusItem::new("Runtime", "Gallery")
                .icon(IconName::Terminal)
                .tone(Tone::Info),
        )
        .left(StatusItem::new("Focus", "Terminal").tone(Tone::Secondary))
        .left(StatusItem::new(
            "Task",
            format!("{task_position}/{task_count}"),
        ))
        .left(StatusItem::new("Worktree", worktree).tone(task_tone))
        .right(StatusItem::new("Session", session_label).tone(session_tone))
        .right(StatusItem::new("Changes", changed.to_string()).tone(Tone::Secondary))
        .right(StatusItem::new("Review", review.to_string()).tone(Tone::Warning))
}

#[cfg(test)]
mod tests {
    use gpui::TestApp;
    use relay::SignalVecExt;

    use super::*;

    #[test]
    fn selected_task_memo_tracks_selection_model_and_list_updates() {
        let mut app = TestApp::new();
        let window = app.open_window(|_, cx| {
            relay_uikit::theme::init(cx);
            WorkbenchApp::new(cx)
        });
        let root = window.root();

        let initial = app.update_entity(&root, |app, cx| app.state.selected_task.get(cx));
        assert_eq!(initial.as_ref().map(|task| task.id), Some(1));

        app.update_entity(&root, |app, cx| {
            app.state.active_task.select(cx, 2);
        });
        let selected = app.update_entity(&root, |app, cx| app.state.selected_task.get(cx));
        assert_eq!(selected.as_ref().map(|task| task.id), Some(2));

        app.update_entity(&root, |app, cx| {
            app.state.tasks.update(cx, |tasks| {
                let Some(task) = tasks.iter_mut().find(|task| task.id == 2) else {
                    return false;
                };
                task.title = "Updated GPUI boundary audit".into();
                true
            });
        });
        let updated = app.update_entity(&root, |app, cx| app.state.selected_task.get(cx));
        assert_eq!(
            updated.as_ref().map(|task| task.title.as_str()),
            Some("Updated GPUI boundary audit")
        );

        app.update_entity(&root, |app, cx| {
            app.state.tasks.remove_selected_by(
                cx,
                app.state.active_task.selection().selector(),
                |task| task.id,
            );
        });

        let reselected = app.update_entity(&root, |app, cx| {
            (
                app.state.active_task.get(cx),
                app.state.selected_task.get(cx).map(|task| task.id),
            )
        });
        assert_eq!(reselected, (Some(1), Some(1)));
    }

    #[test]
    fn review_report_reloads_when_task_source_changes() {
        let mut app = TestApp::new();
        let window = app.open_window(|_, cx| {
            relay_uikit::theme::init(cx);
            WorkbenchApp::new(cx)
        });
        let root = window.root();

        let initial = app.update_entity(&root, |app, cx| {
            app.state.review_report.fold_latest(
                cx,
                || ("pending".to_string(), true),
                |report, loading| (report.headline.clone(), loading),
                |error| (error.clone(), false),
            )
        });
        assert_eq!(initial, ("Review summary ready".to_string(), false));

        app.update_entity(&root, |app, cx| {
            app.state.active_task.select(cx, 2);
        });

        let refreshing = app.update_entity(&root, |app, cx| {
            app.state.review_report.fold_latest(
                cx,
                || ("pending".to_string(), true),
                |report, loading| (report.headline.clone(), loading),
                |error| (error.clone(), false),
            )
        });
        assert_eq!(refreshing, ("Review summary ready".to_string(), true));
    }

    #[test]
    fn terminal_output_reloads_when_selection_source_changes() {
        let mut app = TestApp::new();
        let window = app.open_window(|_, cx| {
            relay_uikit::theme::init(cx);
            WorkbenchApp::new(cx)
        });
        let root = window.root();

        let initial = app.update_entity(&root, |app, cx| {
            app.state.terminal_output.fold_latest(
                cx,
                || (None, true),
                |lines, loading| (lines.first().map(|line| line.text.clone()), loading),
                |_error| (None, false),
            )
        });
        assert_eq!(
            initial,
            (Some("$ codex work relay/workbench".to_string()), false)
        );

        app.update_entity(&root, |app, cx| {
            app.state.active_task.select(cx, 2);
            app.state.active_session.select(cx, 12);
        });

        let refreshing = app.update_entity(&root, |app, cx| {
            app.state.terminal_output.fold_latest(
                cx,
                || (None, true),
                |lines, loading| (lines.first().map(|line| line.text.clone()), loading),
                |_error| (None, false),
            )
        });
        assert_eq!(
            refreshing,
            (Some("$ codex work relay/workbench".to_string()), true)
        );
    }
}
