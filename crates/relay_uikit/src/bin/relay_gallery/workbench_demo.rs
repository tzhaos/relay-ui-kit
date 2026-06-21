//! Orca-direction workbench sample assembled from the Relay UI crates.

mod center;
mod context;
mod data;
mod rail;

use gpui::App;
use gpui::{AnyElement, AppContext, Context, Entity, FocusHandle, IntoElement, Render, Window};
use relay::{
    Binding, ReactiveAppExt, Selector, Signal,
    view::{ReactiveView, StateScope, reactive_render},
};
use relay_uikit::patterns::{AppShell, SplitPane, SplitPaneState, StatusBar, StatusItem};
use relay_uikit::{ActiveTheme, TextInputState, Tone, icon::IconName, theme::space};

use center::center_pane;
use context::right_context;
use data::{
    WorkbenchSession, WorkbenchTask, initial_sessions, initial_tasks, selected_session,
    selected_task,
};
use rail::left_rail;

pub struct WorkbenchApp {
    pub state: WorkbenchState,
    #[allow(dead_code)]
    pub scope: StateScope,
}

impl WorkbenchApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let scope = StateScope::new();
        Self {
            state: WorkbenchState::new(cx),
            scope,
        }
    }
}

/// Interactive state for the Workbench page.
pub struct WorkbenchState {
    tasks: Signal<Vec<WorkbenchTask>>,
    sessions: Signal<Vec<WorkbenchSession>>,
    active_task: Selector<u64>,
    active_session: Selector<u64>,
    task_list: Entity<rail::TaskListView>,
    session_list: Entity<context::SessionListView>,
    pub context_tab: Binding<&'static str>,
    pub route: Binding<&'static str>,
    pub filter: Binding<TextInputState>,
    pub filter_focus: FocusHandle,
    pub left_split: Entity<SplitPaneState>,
    pub terminal_split: Entity<SplitPaneState>,
}

impl WorkbenchState {
    pub fn new(cx: &mut Context<WorkbenchApp>) -> Self {
        let tasks = cx.signal(initial_tasks());
        let sessions = cx.signal(initial_sessions());
        let active_task = cx.selector(Some(1));
        let active_session = cx.selector(Some(11));
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
            task_list,
            session_list,
            context_tab: cx.binding("files"),
            route: cx.binding("terminal"),
            filter: cx.binding(TextInputState::new()),
            filter_focus: cx.focus_handle(),
            left_split: cx.new(|_| SplitPaneState::new(space::RAIL_WIDTH)),
            terminal_split: cx.new(|_| SplitPaneState::new(760.0)),
        }
    }
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
    let tasks = state.tasks.get(cx);
    let sessions = state.sessions.get(cx);
    let task = selected_task(&tasks, state.active_task.get(cx));
    let session = selected_session(&sessions, state.active_session.get(cx));
    let worktree = task.map_or("No task", |task| task.worktree);
    let task_tone = task.map_or(Tone::Muted, |task| task.tone);
    let session_label = session.map_or("No session", |session| session.label.as_str());
    let session_tone = session.map_or(Tone::Muted, |session| session.tone);
    let changed = task.map_or(0, |task| task.changed);
    let review = task.map_or(0, |task| task.review);

    StatusBar::new()
        .left(
            StatusItem::new("Runtime", "Gallery")
                .icon(IconName::Terminal)
                .tone(Tone::Info),
        )
        .left(StatusItem::new("Focus", state.route.get(cx)).tone(Tone::Secondary))
        .left(StatusItem::new("Worktree", worktree).tone(task_tone))
        .right(StatusItem::new("Session", session_label).tone(session_tone))
        .right(StatusItem::new("Changes", changed.to_string()).tone(Tone::Secondary))
        .right(StatusItem::new("Review", review.to_string()).tone(Tone::Warning))
}
