use gpui::{Context, Entity, IntoElement, ParentElement, Styled, div, px};
use relay_uikit::patterns::ScrollSurface;
use relay_uikit::{
    Button, ButtonVariant, IconButton, IconName, IconSize, Label, LabelSize, ListItem, Theme, Tone,
    TreeRow, radius,
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
                .child(long_labels())
                .child(long_file_tree()),
        ))
        .child(section(
            cx,
            "Disabled and quiet states",
            strip()
                .child(
                    Button::new("stress-disabled-primary", "Primary Action")
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
        .child(section(cx, "Dense rows", long_list(theme)))
        .child(section(cx, "Scroll surface", scroll_surface_sample(theme)))
}

fn long_labels() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(Label::new(
            "Repair terminal focus after switching between a Codex session and a plain shell in a nested worktree",
        ))
        .child(
            Label::new("Check long review note delivery state with an extremely verbose label that exceeds any reasonable container width constraint set by the layout system")
                .size(LabelSize::Small),
        )
        .child(
            Label::new("short but dense row")
                .size(LabelSize::XSmall),
        )
}

fn long_list(theme: Theme) -> impl IntoElement {
    div()
        .w(px(420.0))
        .flex()
        .flex_col()
        .gap_1()
        .child(
            ListItem::new("stress-dense-1")
                .child(div().truncate().text_sm().text_color(theme.text).child(
                    "Repair terminal focus after switching between a Codex session and a plain shell",
                ))
                .end_slot(div().text_size(px(11.0)).text_color(Theme::light().danger).child("RUNNING")),
        )
        .child(
            ListItem::new("stress-dense-2")
                .child(div().truncate().text_sm().text_color(theme.text).child(
                    "Check long review note delivery state with verbose label text",
                ))
                .end_slot(div().text_size(px(11.0)).text_color(Theme::light().warning).child("WAITING")),
        )
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
                "relay_uikit/src/components/controls/segmented_control_very_long_name.rs",
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
            "components/list/tree_view_long_item_name.rs",
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
