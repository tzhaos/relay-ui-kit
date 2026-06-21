use gpui::{
    Anchor, App, IntoElement, ParentElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use relay_uikit::patterns::{AnchoredOverlay, Pane, PaneSurface, PaneWidth};
use relay_uikit::workbench::{
    FileKind, FileView, LauncherItem, LauncherItemKind, LauncherMenu, MarkdownView,
    TerminalAgentQuickLaunch, TerminalStatusBadge, TerminalSurface, TerminalTab, TerminalToolbar,
    TerminalTranscript,
};
use relay_uikit::{
    Button, IconButton, IconName, PanelHeader, Segment, SegmentedControl, Theme, theme,
};

use super::{
    WorkbenchState,
    data::{
        DEMO_SESSIONS, DemoTask, active_session, active_task, prompt_prefix, session_index_for_key,
        session_lines,
    },
};

pub(super) fn center_pane(
    state: &WorkbenchState,
    theme: Theme,
    cx: &App,
) -> impl IntoElement {
    let active = active_task(state.active_task.get(cx));
    let route = state.route.get(cx);
    let header = PanelHeader::new(active.title)
        .icon(IconName::Terminal)
        .trailing(route_switch(state));

    let body = div()
        .size_full()
        .flex()
        .flex_col()
        .when(route == "terminal", |this| {
            this.child(terminal_body(state, theme, cx))
        })
        .when(route == "preview", |this| this.child(preview_body(active)));

    Pane::new(PaneWidth::Flex, body)
        .surface(PaneSurface::Panel)
        .header(header)
}

fn route_switch(state: &WorkbenchState) -> impl IntoElement {
    SegmentedControl::bound(
        "route",
        vec![
            Segment::new("terminal", "Terminal"),
            Segment::new("preview", "Preview"),
        ],
        state.route.clone(),
    )
}

fn terminal_body(
    state: &WorkbenchState,
    theme: Theme,
    cx: &App,
) -> impl IntoElement {
    let session = active_session(state.active_session.get(cx));

    div()
        .size_full()
        .min_h_0()
        .flex()
        .flex_col()
        .child(terminal_toolbar(state, cx))
        .child(agent_quick_launches(state, theme))
        .child(
            div()
                .h(px(30.0))
                .px_3()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .border_b_1()
                .border_color(theme.border)
                .bg(theme.panel_alt)
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .font_family(theme::mono_family())
                        .text_size(px(12.0))
                        .text_color(theme.text_secondary)
                        .child(session.cwd),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .font_family(theme::mono_family())
                        .text_size(px(12.0))
                        .text_color(theme.text_muted)
                        .child(session.command),
                )
                .child(TerminalStatusBadge::new(session.tone)),
        )
        .child(
            TerminalSurface::new(
                "workbench-terminal-surface",
                TerminalTranscript::new(session_lines(session)).prompt(format!(
                    "{} {}",
                    prompt_prefix(session),
                    session.cwd
                )),
            )
            .connected(true),
        )
}

fn terminal_toolbar(
    state: &WorkbenchState,
    cx: &App,
) -> impl IntoElement {
    let active_session = state.active_session.clone();
    let launcher_open = state.launcher_open.clone();
    let mut toolbar = TerminalToolbar::new();
    for (index, session) in DEMO_SESSIONS.iter().enumerate() {
        let active_session = active_session.clone();
        let launcher_open = launcher_open.clone();
        toolbar = toolbar.tab(
            TerminalTab::new(("terminal-tab", index), session.label)
                .active(index == state.active_session.get(cx))
                .status(session.tone)
                .on_click(move |_event, _window, cx| {
                    active_session.set(cx, index);
                    launcher_open.set(cx, false);
                }),
        );
    }

    toolbar.actions(terminal_actions(state, cx))
}

fn terminal_actions(state: &WorkbenchState, cx: &App) -> impl IntoElement {
    let launcher_open = state.launcher_open.clone();

    div()
        .flex()
        .items_center()
        .gap_1()
        .child(
            AnchoredOverlay::new(
                "terminal-launcher-overlay",
                Button::new("terminal-new", "New")
                    .icon(IconName::Plus)
                    .on_click({
                        let launcher_open = launcher_open.clone();
                        move |_event, _window, cx| {
                            launcher_open.set(cx, !launcher_open.get(cx));
                        }
                    }),
                launcher_menu(state),
            )
            .open(state.launcher_open.get(cx))
            .anchor(Anchor::TopRight)
            .attach(Anchor::BottomRight)
            .on_dismiss({
                let launcher_open = launcher_open.clone();
                move |_window, cx| {
                    launcher_open.set(cx, false);
                }
            }),
        )
        .child(
            IconButton::new("terminal-refresh", IconName::RefreshCw).on_click({
                let launcher_open = launcher_open.clone();
                move |_event, _window, cx| {
                    launcher_open.set(cx, false);
                }
            }),
        )
}

fn launcher_menu(state: &WorkbenchState) -> impl IntoElement {
    let active_session = state.active_session.clone();
    let route = state.route.clone();
    let launcher_open = state.launcher_open.clone();

    LauncherMenu::new(
        "terminal-launcher",
        vec![
            LauncherItem::new("powershell", "New PowerShell", IconName::Terminal)
                .detail("Open a shell in the current workspace")
                .kind(LauncherItemKind::Terminal),
            LauncherItem::new("codex", "Codex Agent", IconName::Bot)
                .detail("Attach Codex to the active task terminal")
                .kind(LauncherItemKind::Agent),
            LauncherItem::new("claude", "Claude Agent", IconName::Bot)
                .detail("Attach Claude to the active task terminal")
                .kind(LauncherItemKind::Agent),
        ],
    )
    .on_select(move |key, _window, cx| {
        active_session.set(cx, session_index_for_key(key));
        route.set(cx, "terminal");
        launcher_open.set(cx, false);
    })
}

fn agent_quick_launches(state: &WorkbenchState, theme: Theme) -> impl IntoElement {
    let active_session = state.active_session.clone();
    let route = state.route.clone();
    let launcher_open = state.launcher_open.clone();

    div()
        .h(px(40.0))
        .px_2()
        .border_b_1()
        .border_color(theme.border)
        .bg(theme.chrome)
        .flex()
        .items_center()
        .gap_2()
        .child(
            TerminalAgentQuickLaunch::new("quick-codex", "Codex", "codex").on_click({
                let active_session = active_session.clone();
                let route = route.clone();
                let launcher_open = launcher_open.clone();
                move |_event: &gpui::ClickEvent, _window: &mut Window, cx: &mut App| {
                    active_session.set(cx, session_index_for_key("codex"));
                    route.set(cx, "terminal");
                    launcher_open.set(cx, false);
                }
            }),
        )
        .child(
            TerminalAgentQuickLaunch::new("quick-claude", "Claude", "claude").on_click({
                let active_session = active_session.clone();
                let route = route.clone();
                let launcher_open = launcher_open.clone();
                move |_event: &gpui::ClickEvent, _window: &mut Window, cx: &mut App| {
                    active_session.set(cx, session_index_for_key("claude"));
                    route.set(cx, "terminal");
                    launcher_open.set(cx, false);
                }
            }),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_xs()
                .text_color(theme.text_muted)
                .child("Shortcuts select a session in this gallery sample."),
        )
}

fn preview_body(active: &DemoTask) -> impl IntoElement {
    div().size_full().p_2().child(FileView::new(
        format!("tasks/{}.md", active.branch.replace('/', "-")),
        FileKind::Markdown,
        MarkdownView::new(task_preview(active)),
    ))
}

fn task_preview(active: &DemoTask) -> String {
    format!(
        r#"# {}

Branch: `{}`

Worktree: `{}`

- Status: {}
- Changed files: {}
- Review notes: {}

```text
terminal session: {}
```
"#,
        active.title,
        active.branch,
        active.worktree,
        active.status,
        active.changed,
        active.review,
        active.session_key
    )
}
