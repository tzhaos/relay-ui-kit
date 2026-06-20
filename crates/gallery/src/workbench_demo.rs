//! Orca-direction workbench sample assembled from the Relay UI crates.

mod center;
mod context;
mod data;
mod rail;

use gpui::{AppContext, Context, Entity, FocusHandle, IntoElement, Render, Window};
use relay_composites::{AppShell, SplitPane, SplitPaneState, StatusBar, StatusItem};
use relay_foundation::{ActiveTheme, TextInputState, Tone, icon::IconName, theme::space};

use center::center_pane;
use context::right_context;
use data::{active_session, active_task};
use rail::left_rail;

pub struct WorkbenchApp {
    pub state: WorkbenchState,
}

impl WorkbenchApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            state: WorkbenchState::new(cx),
        }
    }
}

/// Interactive state for the Workbench page.
pub struct WorkbenchState {
    pub active_task: usize,
    pub active_session: usize,
    pub context_tab: &'static str,
    pub route: &'static str,
    pub filter: TextInputState,
    pub filter_focus: FocusHandle,
    pub launcher_open: bool,
    pub left_split: Entity<SplitPaneState>,
    pub terminal_split: Entity<SplitPaneState>,
}

impl WorkbenchState {
    pub fn new(cx: &mut Context<WorkbenchApp>) -> Self {
        Self {
            active_task: 0,
            active_session: 0,
            context_tab: "files",
            route: "terminal",
            filter: TextInputState::new(),
            filter_focus: cx.focus_handle(),
            launcher_open: false,
            left_split: cx.new(|_| SplitPaneState::new(space::RAIL_WIDTH)),
            terminal_split: cx.new(|_| SplitPaneState::new(760.0)),
        }
    }
}

impl Render for WorkbenchApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.theme();
        let host = cx.entity();
        let state = &self.state;
        let left = left_rail(state, &host, theme);
        let center = center_pane(state, &host, theme);
        let right = right_context(state, &host, window, theme);
        let center_and_context = SplitPane::new("center-context-split", center, right)
            .state(state.terminal_split.clone())
            .min_sizes(560.0, 320.0)
            .first_size(760.0);

        let workbench = SplitPane::new("workbench-left-split", left, center_and_context)
            .state(state.left_split.clone())
            .min_sizes(260.0, 780.0)
            .first_size(space::RAIL_WIDTH);

        AppShell::new(workbench).status_bar(status_bar(state))
    }
}

fn status_bar(state: &WorkbenchState) -> impl IntoElement {
    let task = active_task(state);
    let session = active_session(state);

    StatusBar::new()
        .left(
            StatusItem::new("Runtime", "Gallery")
                .icon(IconName::Terminal)
                .tone(Tone::Info),
        )
        .left(StatusItem::new("Focus", state.route).tone(Tone::Secondary))
        .left(StatusItem::new("Worktree", task.worktree).tone(task.tone))
        .right(StatusItem::new("Session", session.label).tone(session.tone))
        .right(StatusItem::new("Changes", task.changed.to_string()).tone(Tone::Secondary))
        .right(StatusItem::new("Review", task.review.to_string()).tone(Tone::Warning))
}
