use gpui::{Context, Entity, IntoElement, ParentElement, Styled, div, px};
use relay_uikit::patterns::{
    AnchoredOverlay, CommandRow, ConfirmDialog, DropdownMenu, KeybindingActions, KeybindingRow,
    KeybindingShortcut, KeybindingTable, MenuItem, Popover,
};
use relay_uikit::{Button, IconName, Theme};

use super::{
    GalleryScenesApp, GalleryState,
    shared::{branch_controls, scene_stack, section},
    workbench_samples::{command_sample, launcher_sample},
};

pub(super) fn render(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    theme: Theme,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement {
    let palette_body = div()
        .flex()
        .items_start()
        .gap_3()
        .flex_wrap()
        .child(command_sample(state, host, cx))
        .child(launcher_sample(state, host, theme, cx));

    let overlay_body = overlay_sample(state, host, theme, cx);

    let mut stack = scene_stack()
        .child(section(cx, "Command palette", palette_body))
        .child(section(
            cx,
            "Shortcuts",
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(shortcuts_table(state, host))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child(format!("Shortcut event: {}", state.overlay_event.get(cx))),
                ),
        ))
        .child(section(
            cx,
            "Branch controls",
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(branch_controls(
                    host,
                    &state.branch_choice,
                    &state.branch_picker_open,
                    &state.branch_actions_open,
                    &state.branch_event,
                ))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child(format!("Branch event: {}", state.branch_event.get(cx))),
                ),
        ))
        .child(section(cx, "Overlays", overlay_body))
        .child(section(
            cx,
            "Command rows",
            div()
                .max_w(px(560.0))
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    CommandRow::new("cmd-row-terminal", "terminal:new", "New Terminal")
                        .detail("Open a shell session")
                        .icon(IconName::Terminal)
                        .shortcut(KeybindingShortcut::new(["Ctrl", "Shift", "T"]))
                        .selected(state.launcher_choice.get(cx) == "terminal:new"),
                )
                .child(
                    CommandRow::new("cmd-row-agent", "agent:codex", "Launch Codex")
                        .detail("Attach Codex to the active terminal")
                        .icon(IconName::Bot)
                        .shortcut(KeybindingShortcut::new(["Ctrl", "K"]))
                        .selected(state.launcher_choice.get(cx) == "agent:codex"),
                ),
        ));

    if state.confirm_dialog_open.get(cx) {
        stack = stack.child(close_terminal_dialog(state, host));
    }

    stack
}

fn shortcuts_table(state: &GalleryState, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    KeybindingTable::new(vec![
        KeybindingRow::new("New terminal")
            .description("Open a shell session")
            .shortcut(KeybindingShortcut::new(["Ctrl", "Shift", "T"]))
            .action(keybinding_actions(state, host, "New terminal")),
        KeybindingRow::new("Launch Codex")
            .description("Attach an agent to the active terminal")
            .shortcut(KeybindingShortcut::new(["Ctrl", "K"]))
            .action(keybinding_actions(state, host, "Launch Codex")),
        KeybindingRow::new("Filter files")
            .description("Focus the active panel search field")
            .shortcut(KeybindingShortcut::new(["Ctrl", "F"]))
            .action(keybinding_actions(state, host, "Filter files")),
    ])
}

fn keybinding_actions(
    state: &GalleryState,
    _host: &Entity<GalleryScenesApp>,
    command: &'static str,
) -> impl IntoElement {
    KeybindingActions::new(format!("shortcut-actions-{command}"))
        .on_edit(shortcut_event(state, command, "Edit"))
        .on_reset(shortcut_event(state, command, "Reset"))
        .on_clear(shortcut_event(state, command, "Clear"))
}

fn shortcut_event(
    state: &GalleryState,
    command: &'static str,
    action: &'static str,
) -> impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static {
    let event = state.overlay_event.clone();
    move |_event, _window, cx| {
        event.set(cx, format!("{action} shortcut: {command}"));
    }
}

fn overlay_sample(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    theme: Theme,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
    div()
        .relative()
        .min_h(px(132.0))
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .flex_wrap()
                .child(
                    AnchoredOverlay::new(
                        "session-popover-overlay",
                        Button::new("overlay-popover", "Session Info")
                            .icon(IconName::MessageSquareText)
                            .on_click({
                                let popover = state.command_popover_open.clone();
                                let context = state.command_context_open.clone();
                                move |_event, _window, cx| {
                                    let was = popover.get(cx);
                                    popover.set(cx, !was);
                                    context.set(cx, false);
                                }
                            }),
                        session_popover(theme),
                    )
                    .open(state.command_popover_open.get(cx))
                    .on_dismiss({
                        let popover = state.command_popover_open.clone();
                        move |_window, cx| {
                            popover.set(cx, false);
                        }
                    }),
                )
                .child(terminal_dropdown_menu(state, host, cx))
                .child(
                    Button::new("overlay-confirm", "Close Terminal")
                        .danger()
                        .icon(IconName::Archive)
                        .on_click({
                            let confirm = state.confirm_dialog_open.clone();
                            let popover = state.command_popover_open.clone();
                            let context = state.command_context_open.clone();
                            move |_event, _window, cx| {
                                confirm.set(cx, true);
                                popover.set(cx, false);
                                context.set(cx, false);
                            }
                        }),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child(format!("Overlay event: {}", state.overlay_event.get(cx))),
        )
}

fn session_popover(theme: Theme) -> impl IntoElement {
    Popover::new("session-popover")
        .title("Active terminal")
        .icon(IconName::Terminal)
        .width(300.0)
        .child(
            div()
                .text_sm()
                .line_height(px(18.0))
                .text_color(theme.text_secondary)
                .child("The selected terminal owns the shell state and agent attachment."),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child("Popover content is anchored, elevated, and dismissible by host state."),
        )
}

fn terminal_dropdown_menu(
    state: &GalleryState,
    _host: &Entity<GalleryScenesApp>,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
    DropdownMenu::new(
        "terminal-context-menu",
        Button::new("overlay-context", "Terminal Menu")
            .icon(IconName::Ellipsis)
            .on_click({
                let context = state.command_context_open.clone();
                let popover = state.command_popover_open.clone();
                move |_event, _window, cx| {
                    let was = context.get(cx);
                    context.set(cx, !was);
                    popover.set(cx, false);
                }
            }),
        vec![
            MenuItem::header("Terminal"),
            MenuItem::new("Split terminal")
                .icon(IconName::PanelLeft)
                .submenu_items(vec![
                    menu_action(state, "Split right", IconName::ArrowRight),
                    menu_action(state, "Split down", IconName::LayoutGrid),
                ]),
            menu_action(state, "Rename session", IconName::Settings),
            MenuItem::separator(),
            menu_action(state, "Close session", IconName::Archive).danger(),
        ],
    )
    .open(state.command_context_open.get(cx))
    .min_width(220.0)
    .on_dismiss({
        let context = state.command_context_open.clone();
        move |_window, cx| {
            context.set(cx, false);
        }
    })
}

fn menu_action(
    state: &GalleryState,
    label: &'static str,
    icon: IconName,
) -> MenuItem {
    MenuItem::new(label).icon(icon).on_click({
        let context = state.command_context_open.clone();
        let event = state.overlay_event.clone();
        move |_event, _window, cx| {
            context.set(cx, false);
            event.set(cx, label.to_string());
        }
    })
}

fn close_terminal_dialog(
    state: &GalleryState,
    _host: &Entity<GalleryScenesApp>,
) -> impl IntoElement {
    ConfirmDialog::new(
        "close-terminal-dialog",
        "Close terminal?",
        "The terminal view will be removed from this workspace. Running commands should be stopped by the host before closing.",
    )
    .danger(true)
    .confirm_label("Close")
    .cancel_label("Keep Open")
    .on_cancel({
        let confirm = state.confirm_dialog_open.clone();
        let event = state.overlay_event.clone();
        move |_event, _window, cx| {
            confirm.set(cx, false);
            event.set(cx, "Close cancelled".into());
        }
    })
    .on_confirm({
        let confirm = state.confirm_dialog_open.clone();
        let event = state.overlay_event.clone();
        move |_event, _window, cx| {
            confirm.set(cx, false);
            event.set(cx, "Close confirmed".into());
        }
    })
}
