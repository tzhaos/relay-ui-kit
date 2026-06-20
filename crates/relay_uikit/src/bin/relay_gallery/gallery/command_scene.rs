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
    let mut stack = scene_stack()
        .child(section(
            cx,
            "Command palette",
            div()
                .flex()
                .items_start()
                .gap_3()
                .flex_wrap()
                .child(command_sample(state, host))
                .child(launcher_sample(state, host, theme)),
        ))
        .child(section(
            cx,
            "Shortcuts",
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(shortcuts_table(host))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child(format!("Shortcut event: {}", state.overlay_event)),
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
                    state.branch_choice,
                    state.branch_picker_open,
                    state.branch_actions_open,
                ))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child(format!("Branch event: {}", state.branch_event)),
                ),
        ))
        .child(section(cx, "Overlays", overlay_sample(state, host, theme)))
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
                        .selected(state.launcher_choice == "terminal:new"),
                )
                .child(
                    CommandRow::new("cmd-row-agent", "agent:codex", "Launch Codex")
                        .detail("Attach Codex to the active terminal")
                        .icon(IconName::Bot)
                        .shortcut(KeybindingShortcut::new(["Ctrl", "K"]))
                        .selected(state.launcher_choice == "agent:codex"),
                ),
        ));

    if state.confirm_dialog_open {
        stack = stack.child(close_terminal_dialog(host));
    }

    stack
}

fn shortcuts_table(host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    KeybindingTable::new(vec![
        KeybindingRow::new("New terminal")
            .description("Open a shell session")
            .shortcut(KeybindingShortcut::new(["Ctrl", "Shift", "T"]))
            .action(keybinding_actions(host, "New terminal")),
        KeybindingRow::new("Launch Codex")
            .description("Attach an agent to the active terminal")
            .shortcut(KeybindingShortcut::new(["Ctrl", "K"]))
            .action(keybinding_actions(host, "Launch Codex")),
        KeybindingRow::new("Filter files")
            .description("Focus the active panel search field")
            .shortcut(KeybindingShortcut::new(["Ctrl", "F"]))
            .action(keybinding_actions(host, "Filter files")),
    ])
}

fn keybinding_actions(host: &Entity<GalleryScenesApp>, command: &'static str) -> impl IntoElement {
    KeybindingActions::new(format!("shortcut-actions-{command}"))
        .on_edit(shortcut_event(host, command, "Edit"))
        .on_reset(shortcut_event(host, command, "Reset"))
        .on_clear(shortcut_event(host, command, "Clear"))
}

fn shortcut_event(
    host: &Entity<GalleryScenesApp>,
    command: &'static str,
    action: &'static str,
) -> impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static {
    let host = host.clone();
    move |_event, _window, cx| {
        host.update(cx, |this, cx| {
            this.state.overlay_event = format!("{action} shortcut: {command}");
            cx.notify();
        });
    }
}

fn overlay_sample(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    theme: Theme,
) -> impl IntoElement {
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
                                let host = host.clone();
                                move |_event, _window, cx| {
                                    host.update(cx, |this, cx| {
                                        this.state.command_popover_open =
                                            !this.state.command_popover_open;
                                        this.state.command_context_open = false;
                                        cx.notify();
                                    });
                                }
                            }),
                        session_popover(theme),
                    )
                    .open(state.command_popover_open)
                    .on_dismiss({
                        let host = host.clone();
                        move |_window, cx| {
                            host.update(cx, |this, cx| {
                                this.state.command_popover_open = false;
                                cx.notify();
                            });
                        }
                    }),
                )
                .child(terminal_dropdown_menu(state.command_context_open, host))
                .child(
                    Button::new("overlay-confirm", "Close Terminal")
                        .danger()
                        .icon(IconName::Archive)
                        .on_click({
                            let host = host.clone();
                            move |_event, _window, cx| {
                                host.update(cx, |this, cx| {
                                    this.state.confirm_dialog_open = true;
                                    this.state.command_popover_open = false;
                                    this.state.command_context_open = false;
                                    cx.notify();
                                });
                            }
                        }),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child(format!("Overlay event: {}", state.overlay_event)),
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

fn terminal_dropdown_menu(open: bool, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    DropdownMenu::new(
        "terminal-context-menu",
        Button::new("overlay-context", "Terminal Menu")
            .icon(IconName::Ellipsis)
            .on_click({
                let host = host.clone();
                move |_event, _window, cx| {
                    host.update(cx, |this, cx| {
                        this.state.command_context_open = !this.state.command_context_open;
                        this.state.command_popover_open = false;
                        cx.notify();
                    });
                }
            }),
        vec![
            MenuItem::header("Terminal"),
            MenuItem::new("Split terminal")
                .icon(IconName::PanelLeft)
                .submenu_items(vec![
                    menu_action(host, "Split right", IconName::ArrowRight),
                    menu_action(host, "Split down", IconName::LayoutGrid),
                ]),
            menu_action(host, "Rename session", IconName::Settings),
            MenuItem::separator(),
            menu_action(host, "Close session", IconName::Archive).danger(),
        ],
    )
    .open(open)
    .min_width(220.0)
    .on_dismiss({
        let host = host.clone();
        move |_window, cx| {
            host.update(cx, |this, cx| {
                this.state.command_context_open = false;
                cx.notify();
            });
        }
    })
}

fn menu_action(host: &Entity<GalleryScenesApp>, label: &'static str, icon: IconName) -> MenuItem {
    MenuItem::new(label).icon(icon).on_click({
        let host = host.clone();
        move |_event, _window, cx| {
            host.update(cx, |this, cx| {
                this.state.command_context_open = false;
                this.state.overlay_event = label.to_string();
                cx.notify();
            });
        }
    })
}

fn close_terminal_dialog(host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    ConfirmDialog::new(
        "close-terminal-dialog",
        "Close terminal?",
        "The terminal view will be removed from this workspace. Running commands should be stopped by the host before closing.",
    )
    .danger(true)
    .confirm_label("Close")
    .cancel_label("Keep Open")
    .on_cancel({
        let host = host.clone();
        move |_event, _window, cx| {
            host.update(cx, |this, cx| {
                this.state.confirm_dialog_open = false;
                this.state.overlay_event = "Close cancelled".into();
                cx.notify();
            });
        }
    })
    .on_confirm({
        let host = host.clone();
        move |_event, _window, cx| {
            host.update(cx, |this, cx| {
                this.state.confirm_dialog_open = false;
                this.state.overlay_event = "Close confirmed".into();
                cx.notify();
            });
        }
    })
}
