use gpui::{Context, Entity, IntoElement, ParentElement, Styled, div, px};
use relay_foundation::{
    Button, ButtonVariant, IconButton, IconName, IconSize, ScrollSurface, TaskRow, TaskRowData,
    Theme, Tone, TreeRow, radius,
};
use relay_workbench::{
    CodeView, FileKind, FileView, TerminalLine, TerminalLineStyle, TerminalSurface,
    TerminalTranscript,
};

use super::{
    GalleryScenesApp, GalleryState,
    shared::{scene_stack, section, strip},
};

pub(super) fn render(
    _state: &GalleryState,
    _host: &Entity<GalleryScenesApp>,
    theme: Theme,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement {
    scene_stack()
        .child(section(
            cx,
            "Long text",
            div()
                .flex()
                .items_start()
                .gap_3()
                .flex_wrap()
                .child(long_task_rows())
                .child(long_file_tree()),
        ))
        .child(section(
            cx,
            "Disabled and quiet states",
            strip()
                .child(
                    Button::new("stress-disabled-primary", "Launch Agent")
                        .primary()
                        .icon(IconName::Play)
                        .disabled(true),
                )
                .child(
                    Button::new("stress-disabled-secondary", "Archive")
                        .variant(ButtonVariant::Secondary)
                        .icon(IconName::Archive)
                        .disabled(true),
                )
                .child(
                    Button::new("stress-disabled-ghost", "Refresh")
                        .ghost()
                        .icon(IconName::RefreshCw)
                        .disabled(true),
                ),
        ))
        .child(section(
            cx,
            "Disabled icon buttons",
            strip()
                .child(
                    IconButton::new("stress-ib-disabled", IconName::Plus)
                        .size(IconSize::Small)
                        .disabled(true),
                )
                .child(
                    IconButton::new("stress-ib-active-disabled", IconName::PanelLeft)
                        .active(true)
                        .size(IconSize::Small)
                        .disabled(true),
                )
                .child(
                    IconButton::new("stress-ib-active", IconName::Settings)
                        .active(true)
                        .size(IconSize::Small),
                ),
        ))
        .child(section(cx, "Scroll surface", scroll_surface_sample(theme)))
        .child(section(
            cx,
            "Terminal scrollback",
            div()
                .h(px(280.0))
                .border_1()
                .border_color(theme.border)
                .rounded(px(radius::LG))
                .overflow_hidden()
                .child(TerminalSurface::new(
                    "stress-terminal-surface",
                    TerminalTranscript::new(stress_terminal_lines()).prompt("relay>"),
                )),
        ))
        .child(section(
            cx,
            "Code overflow",
            div().h(px(260.0)).child(
                FileView::new(
                    "crates/gallery/src/gallery/stress_scene.rs",
                    FileKind::Code,
                    CodeView::new(STRESS_CODE).language("rust"),
                )
                .detail("long line"),
            ),
        ))
}

fn scroll_surface_sample(theme: Theme) -> impl IntoElement {
    div().h(px(180.0)).child(ScrollSurface::new(
        "stress-scroll-surface",
        div()
            .flex()
            .flex_col()
            .gap(px(1.0))
            .children((0..24).map(move |index| {
                div()
                    .h(px(28.0))
                    .px_2()
                    .flex()
                    .items_center()
                    .justify_between()
                    .rounded(px(radius::MD))
                    .bg(if index % 2 == 0 {
                        theme.panel
                    } else {
                        theme.panel_alt
                    })
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.text_secondary)
                            .child(format!("Session history row {index:02}")),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(theme.text_muted)
                            .child(if index % 3 == 0 { "active" } else { "idle" }),
                    )
            })),
    ))
}

fn long_task_rows() -> impl IntoElement {
    div()
        .w(px(420.0))
        .flex()
        .flex_col()
        .gap_1()
        .child(TaskRow::new(
            "stress-task-long",
            TaskRowData {
                title: "Repair terminal focus after switching between a Codex session and a plain shell in a nested worktree".into(),
                status_label: "RUNNING".into(),
                status_tone: Tone::Accent,
                branch: Some("feature/terminal-focus-after-agent-switching".into()),
                changed: 128,
                review: 12,
            },
        ).selected(true))
        .child(TaskRow::new(
            "stress-task-muted",
            TaskRowData {
                title: "Check long review note delivery state".into(),
                status_label: "WAITING".into(),
                status_tone: Tone::Warning,
                branch: Some("review/very-long-review-delivery-state".into()),
                changed: 42,
                review: 9,
            },
        ))
}

fn long_file_tree() -> impl IntoElement {
    div()
        .w(px(420.0))
        .flex()
        .flex_col()
        .child(
            TreeRow::new("stress-tree-root", IconName::Folder, "crates")
                .expandable(true)
                .depth(0),
        )
        .child(
            TreeRow::new(
                "stress-tree-deep",
                IconName::Folder,
                "relay_workbench/src/terminal/session/history/very/deep/path",
            )
            .depth(1),
        )
        .child(
            TreeRow::new(
                "stress-tree-file",
                IconName::FileText,
                "terminal_session_history_projection_with_extremely_long_name.rs",
            )
            .depth(2)
            .selected(true),
        )
        .child(TreeRow::new(
            "stress-tree-diff",
            IconName::FileDiff,
            "workbench/context/diff/review/comment_delivery.rs",
        ))
}

fn stress_terminal_lines() -> Vec<TerminalLine> {
    (0..14)
        .map(|index| {
            let style = match index % 4 {
                0 => TerminalLineStyle::Input,
                1 => TerminalLineStyle::Output,
                2 => TerminalLineStyle::Success,
                _ => TerminalLineStyle::Muted,
            };
            TerminalLine::new(format!(
                "line {index:02}: terminal scrollback keeps row height stable while content changes"
            ))
            .style(style)
        })
        .collect()
}

const STRESS_CODE: &str = r#"pub fn absurdly_long_terminal_command_preview() {
    let command = "codex --worktree F:/Workspace/Relay/.worktrees/terminal-focus-after-agent-switching --agent codex --review-diff --keep-terminal-focus --really-long-argument-name";
    println!("{command}");
}
"#;
