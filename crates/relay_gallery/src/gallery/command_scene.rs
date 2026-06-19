use gpui::{Context, Entity, IntoElement, ParentElement, Styled, div, px};
use relay_ui_kit::{
    CommandRow, IconButton, IconName, KeybindingRow, KeybindingTable, KeyboardShortcut, Theme,
};

use super::{
    GalleryScenesApp, GalleryState,
    product_samples::{command_sample, launcher_sample},
    shared::{branch_controls, scene_stack, section},
};

pub(super) fn render(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    theme: Theme,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement {
    scene_stack()
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
            KeybindingTable::new(vec![
                KeybindingRow::new("New terminal")
                    .description("Open a shell session")
                    .shortcut(KeyboardShortcut::new(["Ctrl", "Shift", "T"]))
                    .action(IconButton::new(
                        "keybinding-terminal-edit",
                        IconName::Settings,
                    )),
                KeybindingRow::new("Launch Codex")
                    .description("Attach an agent to the active terminal")
                    .shortcut(KeyboardShortcut::new(["Ctrl", "K"]))
                    .action(IconButton::new("keybinding-codex-edit", IconName::Settings)),
                KeybindingRow::new("Filter files")
                    .description("Focus the active panel search field")
                    .shortcut(KeyboardShortcut::new(["Ctrl", "F"]))
                    .action(IconButton::new(
                        "keybinding-filter-edit",
                        IconName::Settings,
                    )),
            ]),
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
                        .shortcut(KeyboardShortcut::new(["Ctrl", "Shift", "T"]))
                        .selected(state.launcher_choice == "terminal:new"),
                )
                .child(
                    CommandRow::new("cmd-row-agent", "agent:codex", "Launch Codex")
                        .detail("Attach Codex to the active terminal")
                        .icon(IconName::Bot)
                        .shortcut(KeyboardShortcut::new(["Ctrl", "K"]))
                        .selected(state.launcher_choice == "agent:codex"),
                ),
        ))
}
