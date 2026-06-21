use gpui::{Context, Div, Entity, IntoElement, ParentElement, Styled, Window, div, px};
use relay_uikit::patterns::{
    AppShell, CommandPalette, CommandRow, KeybindingShortcut, Pane, PaneSurface, PaneWidth,
    SplitPane, StatusBar, StatusItem, TitleBar, WorkspaceBreadcrumb,
};
use relay_uikit::workbench::{
    LauncherItem, LauncherItemKind, LauncherMenu, TerminalAgentQuickLaunch, TerminalLine,
    TerminalLineStyle, TerminalSessionRow, TerminalStatusBadge, TerminalSurface, TerminalTab,
    TerminalToolbar, TerminalTranscript,
};
use relay_uikit::{
    Button, IconName, NavRow, PanelHeader, TextInput, TextInputAction, Theme, Tone, TreeRow, radius,
};

use super::{GalleryScenesApp, GalleryState};

pub(super) fn shell_sample(
    state: &GalleryState,
    _host: &Entity<GalleryScenesApp>,
    cx: &mut Context<GalleryScenesApp>,
) -> Div {
    let split_size = state.shell_split.read(cx).first_size();
    let left = Pane::new(
        PaneWidth::Flex,
        div()
            .p_2()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                NavRow::new("shell-nav-tasks", IconName::ListChecks, "Tasks")
                    .selected(true)
                    .count(4),
            )
            .child(NavRow::new(
                "shell-nav-terminals",
                IconName::Terminal,
                "Terminals",
            ))
            .child(TreeRow::new("shell-tree", IconName::GitBranch, "ui/workbench-shell").depth(1)),
    )
    .surface(PaneSurface::Chrome)
    .header(PanelHeader::new("Rail").icon(IconName::PanelLeft));

    let center = Pane::new(
        PaneWidth::Flex,
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(TerminalSurface::new(
                "gallery-shell-terminal",
                TerminalTranscript::new(vec![
                    TerminalLine::new("$ cargo build -p relay_uikit --bin relay_gallery")
                        .style(TerminalLineStyle::Input),
                    TerminalLine::new("Finished dev build").style(TerminalLineStyle::Success),
                ])
                .prompt("relay>"),
            )),
    )
    .surface(PaneSurface::Panel)
    .header(PanelHeader::new("Terminal").icon(IconName::Terminal));

    let split = SplitPane::new("gallery-shell-split", left, center)
        .state(state.shell_split.clone())
        .first_size(260.0)
        .min_sizes(220.0, 420.0);

    div()
        .h(px(310.0))
        .overflow_hidden()
        .rounded(px(radius::LG))
        .border_1()
        .border_color(Theme::light().border)
        .child(
            AppShell::new(split)
                .title_bar(
                    TitleBar::new("Relay")
                        .project("Gallery")
                        .center(WorkspaceBreadcrumb::new(vec![
                            "Relay".into(),
                            "UI Kit".into(),
                            "Shell".into(),
                        ]))
                        .actions(Button::new("shell-action", "Open").on_click({
                            let seg_tab = state.seg_tab.clone();
                            move |_event, _window, cx| {
                                seg_tab.set(cx, "files");
                            }
                        }))
                        .window_controls(false),
                )
                .status_bar(
                    StatusBar::new()
                        .left(StatusItem::new("Runtime", "Gallery").tone(Tone::Info))
                        .right(StatusItem::new("Split", format!("{split_size:.0}px"))),
                ),
        )
}

pub(super) fn terminal_sample(
    state: &GalleryState,
    _host: &Entity<GalleryScenesApp>,
    theme: Theme,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
    let active = state.terminal_session.get(cx);
    let set_session = |key: &'static str| {
        let terminal_session = state.terminal_session.clone();
        move |_event: &gpui::ClickEvent, _window: &mut Window, cx: &mut gpui::App| {
            terminal_session.set(cx, key);
        }
    };

    div()
        .h(px(300.0))
        .overflow_hidden()
        .rounded(px(radius::LG))
        .border_1()
        .border_color(theme.border)
        .flex()
        .flex_col()
        .child(
            TerminalToolbar::new()
                .tab(
                    TerminalTab::new("sample-tab-shell", "PowerShell")
                        .active(active == "shell")
                        .status(Tone::Info)
                        .on_click(set_session("shell")),
                )
                .tab(
                    TerminalTab::new("sample-tab-codex", "Codex")
                        .active(active == "codex")
                        .status(Tone::Accent)
                        .on_click(set_session("codex")),
                )
                .actions(TerminalStatusBadge::new(if active == "codex" {
                    Tone::Accent
                } else {
                    Tone::Info
                })),
        )
        .child(
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
                    TerminalAgentQuickLaunch::new("sample-codex", "Codex", "codex")
                        .on_click(set_session("codex")),
                )
                .child(
                    TerminalAgentQuickLaunch::new("sample-claude", "Claude", "claude")
                        .on_click(set_session("claude")),
                ),
        )
        .child(
            TerminalSurface::new(
                "gallery-terminal-sample",
                TerminalTranscript::new(terminal_sample_lines(active)).prompt(format!("{active}>")),
            )
            .connected(true),
        )
}

fn terminal_sample_lines(active: &'static str) -> Vec<TerminalLine> {
    match active {
        "codex" => vec![
            TerminalLine::new("$ codex").style(TerminalLineStyle::Input),
            TerminalLine::new("Agent session attached to active terminal.")
                .style(TerminalLineStyle::Success),
        ],
        "claude" => vec![
            TerminalLine::new("$ claude").style(TerminalLineStyle::Input),
            TerminalLine::new("Agent session selected from quick launch.")
                .style(TerminalLineStyle::Output),
        ],
        _ => vec![
            TerminalLine::new("$ pwsh").style(TerminalLineStyle::Input),
            TerminalLine::new("Workspace shell is ready.").style(TerminalLineStyle::Output),
        ],
    }
}

pub(super) fn command_sample(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
    let command_handler = {
        let launcher_choice = state.launcher_choice.clone();
        let terminal_session = state.terminal_session.clone();
        move |key: &'static str, _window: &mut Window, cx: &mut gpui::App| {
            launcher_choice.set(cx, key);
            if key == "agent:codex" {
                terminal_session.set(cx, "codex");
            }
        }
    };

    let command_handler_a = command_handler.clone();
    let command_handler_b = command_handler.clone();
    let command_handler_c = command_handler;

    CommandPalette::new("Run Command")
        .query(
            TextInput::new(
                "command-query",
                state.search_focus.clone(),
                &state.search_input,
            )
            .placeholder("Search commands")
            .leading_icon(IconName::Search)
            .on_key({
                let host = host.clone();
                move |event, _window, cx| {
                    host.update(cx, |this, cx| {
                        match this.state.search_input.handle_key(event) {
                            TextInputAction::Cancel => {
                                this.state.search_input.clear();
                                cx.notify();
                            }
                            action if action.should_notify() => cx.notify(),
                            _ => {}
                        }
                    });
                }
            }),
        )
        .row(
            CommandRow::new("cmd-terminal", "terminal:new", "New Terminal")
                .detail("Open a shell session in the active workspace")
                .icon(IconName::Terminal)
                .shortcut(KeybindingShortcut::new(["Ctrl", "Shift", "T"]))
                .selected(state.launcher_choice.get(cx) == "terminal:new")
                .on_select(command_handler_a),
        )
        .row(
            CommandRow::new("cmd-codex", "agent:codex", "Launch Codex")
                .detail("Attach Codex to the selected task terminal")
                .icon(IconName::Bot)
                .shortcut(KeybindingShortcut::new(["Ctrl", "K"]))
                .selected(state.launcher_choice.get(cx) == "agent:codex")
                .on_select(command_handler_b),
        )
        .row(
            CommandRow::new("cmd-settings", "settings:agent", "Agent Settings")
                .detail("Open the agent configuration surface")
                .icon(IconName::Settings)
                .selected(state.launcher_choice.get(cx) == "settings:agent")
                .on_select(command_handler_c),
        )
        .footer(
            div()
                .text_xs()
                .text_color(Theme::light().text_muted)
                .child(format!("Selected: {}", state.launcher_choice.get(cx))),
        )
}

pub(super) fn launcher_sample(
    state: &GalleryState,
    _host: &Entity<GalleryScenesApp>,
    theme: Theme,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
    div()
        .w(px(340.0))
        .flex()
        .flex_col()
        .gap_2()
        .child(
            LauncherMenu::new(
                "component-launcher",
                vec![
                    LauncherItem::new("powershell", "PowerShell", IconName::Terminal)
                        .detail("Workspace shell")
                        .kind(LauncherItemKind::Terminal),
                    LauncherItem::new("codex", "Codex Agent", IconName::Bot)
                        .detail("Attach to active task")
                        .kind(LauncherItemKind::Agent),
                    LauncherItem::new("claude", "Claude Agent", IconName::Bot)
                        .detail("Attach to active task")
                        .kind(LauncherItemKind::Agent),
                ],
            )
            .on_select({
                let launcher_choice = state.launcher_choice.clone();
                let terminal_session = state.terminal_session.clone();
                move |key, _window, cx| {
                    launcher_choice.set(cx, key);
                    terminal_session.set(cx, match key {
                        "codex" => "codex",
                        "claude" => "claude",
                        _ => "shell",
                    });
                }
            }),
        )
        .child(
            TerminalSessionRow::new(
                "launcher-session",
                format!("Selected {}", state.launcher_choice.get(cx)),
                "Session history row",
            )
            .status(match state.launcher_choice.get(cx) {
                "codex" => Tone::Accent,
                "claude" => Tone::Warning,
                _ => Tone::Info,
            })
            .active(true),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child("Launcher rows update selection state in the gallery."),
        )
}
