use gpui::{
    AnyElement, AnyView, App, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
    div, px,
};
use relay::{
    KeyedSubViews, OrderedSelectionModel, ReactiveView, Selector, Signal, SignalVecExt,
    view::reactive_render,
};
use relay_uikit::patterns::{
    Pane, PaneToolbar, SessionRow, TopToolbar, WorkspaceBreadcrumb,
    display::KeyValue,
    navigation::{Tab, Tabs},
};
use relay_uikit::{Button, IconButton, IconName, Theme, Tone};

use super::{WorkbenchApp, WorkbenchContextTab, WorkbenchState, data::WorkbenchSession};

pub(super) fn right_context(
    state: &WorkbenchState,
    _window: &Window,
    theme: Theme,
    cx: &mut Context<WorkbenchApp>,
) -> impl IntoElement + use<> {
    let tab = state.context_tab.get(cx);

    Pane::context(
        div()
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .child(Tabs::bound(
                "workbench-context-tabs",
                vec![
                    Tab::new(WorkbenchContextTab::Files, "Files").icon(IconName::FileText),
                    Tab::new(WorkbenchContextTab::Sessions, "Sessions").icon(IconName::Terminal),
                    Tab::new(WorkbenchContextTab::Review, "Review")
                        .icon(IconName::MessageSquareText),
                ],
                state.context_tab.clone(),
            ))
            .child(div().flex_1().min_h_0().p_3().child(match tab {
                WorkbenchContextTab::Sessions => cached_session_list(state.session_list.clone()),
                WorkbenchContextTab::Review => review_panel(state, theme, cx).into_any_element(),
                WorkbenchContextTab::Files => files_panel(state, theme, cx).into_any_element(),
            })),
    )
    .header(
        div()
            .h(px(40.0))
            .px_3()
            .border_b_1()
            .border_color(theme.border)
            .child(
                TopToolbar::new()
                    .leading(WorkspaceBreadcrumb::new(vec![
                        "Context".into(),
                        tab.label().into(),
                    ]))
                    .trailing(PaneToolbar::new().action(IconButton::new(
                        "workbench-context-settings",
                        IconName::Settings,
                    ))),
            ),
    )
}

fn cached_session_list(list: Entity<SessionListView>) -> AnyElement {
    let view: AnyView = list.into();
    view.cached(gpui::StyleRefinement::default().w_full())
        .into_any_element()
}

fn files_panel(
    state: &WorkbenchState,
    theme: Theme,
    cx: &mut Context<WorkbenchApp>,
) -> impl IntoElement + use<> {
    let task = state.selected_task.get(cx);

    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .rounded(px(8.0))
                .border_1()
                .border_color(theme.border)
                .bg(theme.panel)
                .p_2()
                .flex()
                .flex_col()
                .gap_1()
                .child(KeyValue::new(
                    "Task",
                    task.as_ref().map_or("None", |task| task.title.as_str()),
                ))
                .child(
                    KeyValue::new("Branch", task.as_ref().map_or("None", |task| task.branch))
                        .icon(IconName::GitBranch),
                )
                .child(
                    KeyValue::new(
                        "Worktree",
                        task.as_ref().map_or("None", |task| task.worktree),
                    )
                    .icon(IconName::Folder),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child("Task metadata is read from the same selector-backed state as the rail."),
        )
}

fn review_panel(
    state: &WorkbenchState,
    theme: Theme,
    cx: &mut Context<WorkbenchApp>,
) -> impl IntoElement + use<> {
    let task = state.selected_task.get(cx);
    let changed = task.as_ref().map_or(0, |task| task.changed);
    let (headline, detail, notes, loading, tone) = state.review_report.fold_latest(
        cx,
        || {
            (
                "Review pending".to_string(),
                "Start a review refresh to collect diagnostics.".to_string(),
                0,
                true,
                Tone::Muted,
            )
        },
        |report, loading| {
            (
                report.headline.clone(),
                report.detail.clone(),
                report.notes,
                loading,
                report.tone,
            )
        },
        |error| {
            (
                "Review refresh failed".to_string(),
                error.clone(),
                0,
                false,
                Tone::Danger,
            )
        },
    );

    div()
        .rounded(px(8.0))
        .border_1()
        .border_color(theme.border)
        .bg(theme.panel)
        .p_3()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    div()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .truncate()
                                .text_sm()
                                .text_color(tone.fg(&theme))
                                .child(headline),
                        )
                        .child(
                            div()
                                .truncate()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child(detail),
                        ),
                )
                .child(
                    Button::new(
                        "workbench-review-refresh",
                        if loading { "Refreshing" } else { "Refresh" },
                    )
                    .ghost()
                    .icon(IconName::RefreshCw)
                    .disabled(loading)
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.state.refresh_review_report(cx);
                    })),
                ),
        )
        .child(KeyValue::new("Review notes", notes.to_string()).icon(IconName::MessageSquareText))
        .child(KeyValue::new("Changed files", changed.to_string()).icon(IconName::FileDiff))
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child("Review diagnostics keep the last ready report visible while refreshing."),
        )
}

pub(super) struct SessionListView {
    sessions: Signal<Vec<WorkbenchSession>>,
    rows: KeyedSubViews<u64, SessionRowView>,
    selection: OrderedSelectionModel<u64>,
}

impl SessionListView {
    pub(super) fn new(
        cx: &mut Context<Self>,
        sessions: Signal<Vec<WorkbenchSession>>,
        selection: OrderedSelectionModel<u64>,
    ) -> Self {
        relay::init(cx);
        Self {
            sessions,
            rows: KeyedSubViews::new(),
            selection,
        }
    }

    fn activate_next(&self, cx: &mut App) {
        let _ = self.selection.select_next(cx);
    }

    fn remove_active(&self, cx: &mut App) {
        self.sessions
            .remove_selected_by(cx, self.selection.selection().selector(), |session| {
                session.id
            });
    }
}

impl ReactiveView for SessionListView {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let sessions = self.sessions.get(cx);
        let selector = self.selection.selection().selector().clone();
        self.rows.sync_with_selector(
            cx,
            self.selection.selection().selector(),
            sessions,
            |session| session.id,
            move |session, _cx| SessionRowView::new(session, selector.clone()),
            |session, row, _cx| row.update_session(session),
        );

        div()
            .w_full()
            .min_h_0()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("workbench-session-next", "Next")
                            .ghost()
                            .icon(IconName::ArrowRight)
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.activate_next(cx);
                            })),
                    )
                    .child(
                        Button::new("workbench-session-remove", "Remove")
                            .ghost()
                            .icon(IconName::Trash2)
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.remove_active(cx);
                            })),
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

impl Render for SessionListView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

struct SessionRowView {
    session: WorkbenchSession,
    selection: Selector<u64>,
}

impl SessionRowView {
    fn new(session: &WorkbenchSession, selection: Selector<u64>) -> Self {
        Self {
            session: session.clone(),
            selection,
        }
    }

    fn update_session(&mut self, session: &WorkbenchSession) -> bool {
        if self.session == *session {
            false
        } else {
            self.session = session.clone();
            true
        }
    }
}

impl ReactiveView for SessionRowView {
    fn render_state(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> AnyElement {
        SessionRow::new(
            format!("workbench-session-{}", self.session.id),
            self.session.label.clone(),
            self.session.detail.clone(),
        )
        .status(if self.session.connected {
            self.session.tone
        } else {
            Tone::Muted
        })
        .active_by(self.selection.clone(), self.session.id)
        .into_any_element()
    }
}

impl Render for SessionRowView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        reactive_render(self, window, cx)
    }
}

#[cfg(test)]
mod tests {
    use gpui::{EntityId, TestApp};
    use relay::ReactiveAppExt;

    use super::*;
    use crate::workbench_demo::data::initial_sessions;

    fn row_ids(rows: &KeyedSubViews<u64, SessionRowView>) -> Vec<(u64, EntityId)> {
        rows.keyed_iter()
            .map(|(key, view)| (*key, view.entity().entity_id()))
            .collect()
    }

    fn session_ids(list: &SessionListView) -> Vec<u64> {
        list.sessions
            .get_untracked()
            .into_iter()
            .map(|session| session.id)
            .collect()
    }

    #[test]
    fn session_list_reconcile_clears_selection_when_active_removed() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| {
            relay_uikit::theme::init(cx);
            let sessions = cx.signal(initial_sessions());
            let sessions_for_selection = sessions.clone();
            let selection = relay::use_ordered_selection_model(
                cx,
                Some(11),
                move |cx| {
                    sessions_for_selection.read(cx, |sessions| {
                        sessions.iter().map(|session| session.id).collect()
                    })
                },
                relay::SelectionReconcilePolicy::SelectFirst,
            );
            SessionListView::new(cx, sessions, selection)
        });
        let root = window.root();

        window.draw();
        app.update_entity(&root, |list, cx| {
            list.remove_active(cx);
        });
        window.draw();

        let selected = app.update_entity(&root, |list, _cx| list.selection.get_untracked());
        let sessions = app.update_entity(&root, |list, _cx| session_ids(list));
        let rows = app.update_entity(&root, |list, _cx| row_ids(&list.rows));
        assert_eq!(selected, Some(12));
        assert_eq!(sessions, vec![12, 13]);
        assert_eq!(
            rows.iter().map(|(key, _)| *key).collect::<Vec<_>>(),
            sessions
        );
    }
}
