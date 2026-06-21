use gpui::{
    AnyElement, AnyView, App, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
    div, px,
};
use relay::{KeyedSubViews, ReactiveView, Selector, Signal, view::reactive_render};
use relay_uikit::patterns::{
    Pane, PaneToolbar, SessionRow, TopToolbar, WorkspaceBreadcrumb,
    display::KeyValue,
    navigation::{Tab, Tabs},
};
use relay_uikit::{Button, IconButton, IconName, Theme, Tone};

use super::{
    WorkbenchApp, WorkbenchState,
    data::{WorkbenchSession, selected_task},
};

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
                    Tab::new("files", "Files").icon(IconName::FileText),
                    Tab::new("sessions", "Sessions").icon(IconName::Terminal),
                    Tab::new("review", "Review").icon(IconName::MessageSquareText),
                ],
                state.context_tab.clone(),
            ))
            .child(div().flex_1().min_h_0().p_3().child(match tab {
                "sessions" => cached_session_list(state.session_list.clone()),
                "review" => review_panel(state, theme, cx).into_any_element(),
                _ => files_panel(state, theme, cx).into_any_element(),
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
                        tab.to_string(),
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
    let tasks = state.tasks.get(cx);
    let task = selected_task(&tasks, state.active_task.get(cx));

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
                    task.map_or("None", |task| task.title.as_str()),
                ))
                .child(
                    KeyValue::new("Branch", task.map_or("None", |task| task.branch))
                        .icon(IconName::GitBranch),
                )
                .child(
                    KeyValue::new("Worktree", task.map_or("None", |task| task.worktree))
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
    let tasks = state.tasks.get(cx);
    let task = selected_task(&tasks, state.active_task.get(cx));
    let review = task.map_or(0, |task| task.review);
    let changed = task.map_or(0, |task| task.changed);

    div()
        .rounded(px(8.0))
        .border_1()
        .border_color(theme.border)
        .bg(theme.panel)
        .p_3()
        .flex()
        .flex_col()
        .gap_2()
        .child(KeyValue::new("Review notes", review.to_string()).icon(IconName::MessageSquareText))
        .child(KeyValue::new("Changed files", changed.to_string()).icon(IconName::FileDiff))
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child("Review counters update from the selected task without row-local coupling."),
        )
}

pub(super) struct SessionListView {
    sessions: Signal<Vec<WorkbenchSession>>,
    rows: KeyedSubViews<u64, SessionRowView>,
    selection: Selector<u64>,
}

impl SessionListView {
    pub(super) fn new(
        cx: &mut Context<Self>,
        sessions: Signal<Vec<WorkbenchSession>>,
        selection: Selector<u64>,
    ) -> Self {
        relay::init(cx);
        Self {
            sessions,
            rows: KeyedSubViews::new(),
            selection,
        }
    }

    fn activate_next(&self, cx: &mut App) {
        self.sessions.peek(|sessions| {
            self.selection
                .select_next_by(cx, sessions, |session| session.id);
        });
    }

    fn remove_active(&self, cx: &mut App) {
        let Some(selected) = self.selection.get_untracked() else {
            return;
        };

        self.sessions.update(cx, |sessions| {
            let Some(index) = sessions.iter().position(|session| session.id == selected) else {
                return false;
            };
            sessions.remove(index);
            true
        });

        self.sessions.peek(|sessions| {
            self.selection
                .reconcile_keys_by(cx, sessions, |session| session.id);
        });
    }
}

impl ReactiveView for SessionListView {
    fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let sessions = self.sessions.get(cx);
        self.selection
            .reconcile_keys_by(cx, &sessions, |session| session.id);

        let selection = self.selection.clone();
        self.rows.sync(
            cx,
            sessions,
            |session| session.id,
            move |session, _cx| SessionRowView::new(session, selection.clone()),
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
            let selection = cx.selector(Some(11));
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
        assert_eq!(selected, None);
        assert_eq!(sessions, vec![12, 13]);
        assert_eq!(
            rows.iter().map(|(key, _)| *key).collect::<Vec<_>>(),
            sessions
        );
    }
}
