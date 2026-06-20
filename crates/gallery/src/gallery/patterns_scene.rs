use gpui::{Context, Entity, IntoElement, ParentElement, Styled, div, px};
use relay_ui_core::{Button, IconButton, IconName, Label, Theme};
use relay_ui_patterns::{
    PaneToolbar, TopToolbar, WorkspaceBreadcrumb,
    display::KeyValue,
    layout::ListSection,
    navigation::{Tab, Tabs},
    overlay::{ContextMenu, Dialog, MenuItem, Select, SelectOption, TooltipBody},
};

use super::{
    GalleryScenesApp, GalleryState,
    shared::{scene_stack, section, strip},
};

pub(super) fn render(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    theme: Theme,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement {
    let mut stack = scene_stack()
        .child(section(cx, "Layout patterns", layout_patterns(host, theme)))
        .child(section(
            cx,
            "Display patterns",
            display_patterns(host, theme),
        ))
        .child(section(
            cx,
            "Navigation patterns",
            navigation_patterns(state, host),
        ))
        .child(section(
            cx,
            "Overlay patterns",
            overlay_patterns(state, host, theme),
        ));

    if state.pattern_dialog_open {
        stack = stack.child(settings_dialog(host));
    }

    stack
}

fn layout_patterns(host: &Entity<GalleryScenesApp>, theme: Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .h(px(38.0))
                .rounded(px(8.0))
                .border_1()
                .border_color(theme.border)
                .bg(theme.chrome)
                .px_2()
                .child(
                    TopToolbar::new()
                        .leading(Label::new("TopToolbar").strong())
                        .center(WorkspaceBreadcrumb::new(vec![
                            "Relay".into(),
                            "Patterns".into(),
                            "Shell".into(),
                        ]))
                        .trailing(
                            PaneToolbar::new()
                                .action(IconButton::new("toolbar-search", IconName::Search))
                                .action(IconButton::new("toolbar-refresh", IconName::RefreshCw))
                                .action(IconButton::new("toolbar-more", IconName::Ellipsis)),
                        ),
                ),
        )
        .child(
            strip()
                .child(
                    Button::new("layout-left-action", "Focus")
                        .icon(IconName::PanelLeft)
                        .on_click(pattern_event(host, "Layout toolbar focused")),
                )
                .child(
                    Button::new("layout-right-action", "Refresh")
                        .icon(IconName::RefreshCw)
                        .on_click(pattern_event(host, "Layout toolbar refreshed")),
                ),
        )
}

fn display_patterns(host: &Entity<GalleryScenesApp>, theme: Theme) -> impl IntoElement {
    div()
        .grid()
        .grid_cols(2)
        .gap_4()
        .child(
            ListSection::new("Session metadata")
                .count(4)
                .trailing(
                    Button::new("metadata-copy", "Copy")
                        .ghost()
                        .on_click(pattern_event(host, "Metadata copied")),
                )
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
                        .child(KeyValue::new("Project", "relay-ui-kit").icon(IconName::Folder))
                        .child(KeyValue::new("Branch", "master").icon(IconName::GitBranch))
                        .child(KeyValue::new("Layer", "patterns/display"))
                        .child(KeyValue::new("State", "host-owned")),
                ),
        )
        .child(
            div()
                .rounded(px(8.0))
                .border_1()
                .border_color(theme.border)
                .bg(theme.panel)
                .p_3()
                .flex()
                .flex_col()
                .gap_2()
                .child(Label::new("TooltipBody").strong())
                .child(TooltipBody::new(
                    "Hover and focus labels use this compact body",
                ))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child("Display patterns are composed but still product-neutral."),
                ),
        )
}

fn navigation_patterns(state: &GalleryState, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    div().max_w(px(640.0)).child(
        Tabs::new(
            "patterns-navigation-tabs",
            vec![
                Tab::new("files", "Files").icon(IconName::FileText),
                Tab::new("diff", "Diff").icon(IconName::FileDiff).count(12),
                Tab::new("review", "Review")
                    .icon(IconName::MessageSquareText)
                    .count(3),
            ],
        )
        .active(state.seg_tab)
        .on_select({
            let host = host.clone();
            move |key, _window, cx| {
                host.update(cx, |this, cx| {
                    this.state.seg_tab = key;
                    this.state.overlay_event = format!("Navigation tab: {key}");
                    cx.notify();
                });
            }
        }),
    )
}

fn overlay_patterns(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    theme: Theme,
) -> impl IntoElement {
    div()
        .relative()
        .min_h(px(188.0))
        .flex()
        .items_start()
        .gap_4()
        .flex_wrap()
        .child(
            div().w(px(260.0)).child(
                Select::new(
                    "patterns-overlay-select",
                    state.theme_choice,
                    vec![
                        SelectOption::new("system", "System").detail("Follow OS appearance"),
                        SelectOption::new("light", "Light"),
                        SelectOption::new("dark", "Dark"),
                    ],
                )
                .open(state.pattern_select_open)
                .on_toggle({
                    let host = host.clone();
                    move |_event, _window, cx| {
                        host.update(cx, |this, cx| {
                            this.state.pattern_select_open = !this.state.pattern_select_open;
                            cx.notify();
                        });
                    }
                })
                .on_select({
                    let host = host.clone();
                    move |key, _window, cx| {
                        host.update(cx, |this, cx| {
                            this.state.theme_choice = key;
                            this.state.pattern_select_open = false;
                            this.state.overlay_event = format!("Select: {key}");
                            cx.notify();
                        });
                    }
                })
                .on_dismiss({
                    let host = host.clone();
                    move |_window, cx| {
                        host.update(cx, |this, cx| {
                            this.state.pattern_select_open = false;
                            cx.notify();
                        });
                    }
                }),
            ),
        )
        .child(
            Button::new("patterns-dialog-open", "Open Dialog")
                .icon(IconName::Settings)
                .on_click({
                    let host = host.clone();
                    move |_event, _window, cx| {
                        host.update(cx, |this, cx| {
                            this.state.pattern_dialog_open = true;
                            cx.notify();
                        });
                    }
                }),
        )
        .child(
            div()
                .relative()
                .w(px(240.0))
                .h(px(148.0))
                .rounded(px(8.0))
                .border_1()
                .border_color(theme.border)
                .bg(theme.panel)
                .child(
                    div()
                        .p_3()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(Label::new("ContextMenu").strong())
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child("Static overlay sample"),
                        ),
                )
                .child(
                    ContextMenu::new(
                        "patterns-context-menu",
                        vec![
                            MenuItem::header("Terminal"),
                            menu_action(host, "Split right", IconName::ArrowRight),
                            menu_action(host, "Rename", IconName::Settings),
                            MenuItem::separator(),
                            menu_action(host, "Close", IconName::Archive).danger(),
                        ],
                    )
                    .offset(24.0, 64.0)
                    .min_width(190.0),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child(format!("Pattern event: {}", state.overlay_event)),
        )
}

fn settings_dialog(host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    Dialog::new("patterns-settings-dialog", "Pattern settings")
        .description("Dialog uses the overlay layer and host-owned dismiss state.")
        .icon(IconName::Settings)
        .width(420.0)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(KeyValue::new("Layer", "patterns/overlay"))
                .child(KeyValue::new("Dismiss", "backdrop or button"))
                .child(KeyValue::new("Motion", "fade-slide")),
        )
        .footer(
            div()
                .flex()
                .justify_end()
                .gap_2()
                .child(
                    Button::new("patterns-dialog-cancel", "Cancel")
                        .ghost()
                        .on_click(close_dialog(host, "Dialog cancelled")),
                )
                .child(
                    Button::new("patterns-dialog-save", "Save")
                        .primary()
                        .on_click(close_dialog(host, "Dialog saved")),
                ),
        )
        .on_dismiss(close_dialog(host, "Dialog dismissed"))
}

fn menu_action(host: &Entity<GalleryScenesApp>, label: &'static str, icon: IconName) -> MenuItem {
    MenuItem::new(label)
        .icon(icon)
        .on_click(pattern_event(host, label))
}

fn pattern_event(
    host: &Entity<GalleryScenesApp>,
    message: &'static str,
) -> impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static {
    let host = host.clone();
    move |_event, _window, cx| {
        host.update(cx, |this, cx| {
            this.state.overlay_event = message.to_string();
            cx.notify();
        });
    }
}

fn close_dialog(
    host: &Entity<GalleryScenesApp>,
    message: &'static str,
) -> impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static {
    let host = host.clone();
    move |_event, _window, cx| {
        host.update(cx, |this, cx| {
            this.state.pattern_dialog_open = false;
            this.state.overlay_event = message.to_string();
            cx.notify();
        });
    }
}
