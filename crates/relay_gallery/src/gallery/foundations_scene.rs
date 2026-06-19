use gpui::{Context, Entity, IntoElement, ParentElement, Styled, div, prelude::FluentBuilder};
use relay_ui_kit::{
    Badge, Button, ButtonVariant, CountBadge, Disclosure, Divider, EmptyState, FilterBar,
    FilterChip, Icon, IconButton, IconName, Label, LabelSize, ListItem, NavRow, SectionedList,
    SectionedListGroup, Segment, SegmentedControl, Stepper, Tab, Tabs, TaskRow, TaskRowData, Theme,
    Tone, ToolbarGroup, TreeNode, TreeRow, TreeView,
};

use super::{
    GalleryScenesApp, GalleryState,
    shared::{dot_label, icon_sample, scene_stack, section, strip},
};

pub(super) fn render(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    theme: Theme,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement {
    scene_stack()
        .child(section(cx, "Buttons", button_samples(host)))
        .child(section(cx, "Icon buttons", icon_button_samples(host)))
        .child(section(
            cx,
            "Steppers and filters",
            stepper_filter_samples(state, host),
        ))
        .child(section(
            cx,
            "Status and icons",
            div()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    strip()
                        .child(dot_label(theme, Tone::Accent, "running"))
                        .child(dot_label(theme, Tone::Warning, "waiting"))
                        .child(dot_label(theme, Tone::Danger, "failed"))
                        .child(dot_label(theme, Tone::Muted, "idle")),
                )
                .child(
                    strip()
                        .child(icon_sample(theme, IconName::Terminal))
                        .child(icon_sample(theme, IconName::Folder))
                        .child(icon_sample(theme, IconName::FileText))
                        .child(icon_sample(theme, IconName::FileDiff))
                        .child(icon_sample(theme, IconName::GitBranch))
                        .child(icon_sample(theme, IconName::Bot))
                        .child(icon_sample(theme, IconName::Search))
                        .child(icon_sample(theme, IconName::Zap))
                        .child(icon_sample(theme, IconName::MessageSquareText)),
                ),
        ))
        .child(section(
            cx,
            "Labels, badges, disclosure",
            label_badge_samples(state, host),
        ))
        .child(section(
            cx,
            "Navigation rows",
            div()
                .flex()
                .items_start()
                .gap_4()
                .flex_wrap()
                .child(nav_rows_sample())
                .child(tree_rows_sample())
                .child(task_rows_sample()),
        ))
        .child(section(
            cx,
            "List foundations",
            list_foundation_samples(state, host, theme),
        ))
        .child(section(
            cx,
            "Tabs and empty state",
            tab_samples(state, host),
        ))
}

fn label_badge_samples(state: &GalleryState, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            strip()
                .child(Label::new("Primary label").strong())
                .child(Label::new("Secondary metadata").secondary())
                .child(Label::new("Muted caption").muted().size(LabelSize::XSmall))
                .child(Label::new("Accent state").tone(Tone::Accent).strong()),
        )
        .child(
            strip()
                .child(Badge::new("running").tone(Tone::Accent).soft())
                .child(
                    Badge::new("review")
                        .tone(Tone::Info)
                        .icon(IconName::FileDiff),
                )
                .child(CountBadge::new(12).tone(Tone::Secondary))
                .child(CountBadge::new(128).max(99).tone(Tone::Warning)),
        )
        .child(
            div()
                .w(gpui::px(360.0))
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    Disclosure::new(
                        "foundation-disclosure",
                        "Recent terminal sessions",
                        state.foundations_disclosure_open,
                    )
                    .detail("host-owned state")
                    .count(3)
                    .on_toggle({
                        let host = host.clone();
                        move |_event, _window, cx| {
                            host.update(cx, |this, cx| {
                                this.state.foundations_disclosure_open =
                                    !this.state.foundations_disclosure_open;
                                cx.notify();
                            });
                        }
                    }),
                )
                .when(state.foundations_disclosure_open, |this| {
                    this.child(
                        div()
                            .ml_6()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(Label::new("PowerShell").secondary())
                            .child(Label::new("Codex attached").tone(Tone::Accent))
                            .child(Label::new("Diff review").muted()),
                    )
                }),
        )
}

fn button_samples(host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    strip()
        .child(
            Button::new("btn-primary", "Launch Agent")
                .primary()
                .icon(IconName::Play)
                .on_click({
                    let host = host.clone();
                    move |_event, _window, cx| {
                        host.update(cx, |this, cx| {
                            this.state.terminal_session = "codex";
                            cx.notify();
                        });
                    }
                }),
        )
        .child(
            Button::new("btn-secondary", "Refresh")
                .icon(IconName::RefreshCw)
                .on_click({
                    let host = host.clone();
                    move |_event, _window, cx| {
                        host.update(cx, |this, cx| {
                            this.state.search_input.clear();
                            cx.notify();
                        });
                    }
                }),
        )
        .child(
            Button::new("btn-ghost", "Archive")
                .ghost()
                .icon(IconName::Archive)
                .on_click({
                    let host = host.clone();
                    move |_event, _window, cx| {
                        host.update(cx, |this, cx| {
                            this.state.auto_archive = !this.state.auto_archive;
                            cx.notify();
                        });
                    }
                }),
        )
        .child(
            Button::new("btn-disabled", "Disabled")
                .variant(ButtonVariant::Secondary)
                .disabled(true),
        )
}

fn icon_button_samples(host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    strip()
        .child(
            ToolbarGroup::new("foundation-toolbar-group")
                .child(
                    IconButton::new("ib-filter", IconName::ListFilter).on_click({
                        let host = host.clone();
                        move |_event, _window, cx| {
                            host.update(cx, |this, cx| {
                                this.state.seg_tab = "files";
                                cx.notify();
                            });
                        }
                    }),
                )
                .child(
                    IconButton::new("ib-refresh", IconName::RefreshCw).on_click({
                        let host = host.clone();
                        move |_event, _window, cx| {
                            host.update(cx, |this, cx| {
                                this.state.search_input.clear();
                                cx.notify();
                            });
                        }
                    }),
                )
                .child(
                    IconButton::new("ib-settings", IconName::Settings).on_click({
                        let host = host.clone();
                        move |_event, _window, cx| {
                            host.update(cx, |this, cx| {
                                this.state.launcher_choice = "settings";
                                cx.notify();
                            });
                        }
                    }),
                ),
        )
        .child(IconButton::new("ib-active", IconName::PanelLeft).active(true))
}

fn stepper_filter_samples(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            FilterBar::new("foundation-filter-bar")
                .child(
                    FilterChip::new("filter-all-sessions", "All sessions")
                        .icon(IconName::ListFilter)
                        .count(4)
                        .selected(state.theme_choice == "system")
                        .dropdown(true)
                        .on_click({
                            let host = host.clone();
                            move |_event, _window, cx| {
                                host.update(cx, |this, cx| {
                                    this.state.theme_choice = "system";
                                    cx.notify();
                                });
                            }
                        }),
                )
                .child(
                    FilterChip::new("filter-running", "Running")
                        .icon(IconName::CircleDot)
                        .selected(state.theme_choice == "light")
                        .on_click({
                            let host = host.clone();
                            move |_event, _window, cx| {
                                host.update(cx, |this, cx| {
                                    this.state.theme_choice = "light";
                                    cx.notify();
                                });
                            }
                        }),
                )
                .child(
                    FilterChip::new("filter-projects", "All projects")
                        .icon(IconName::Folder)
                        .selected(state.theme_choice == "dark")
                        .dropdown(true)
                        .on_click({
                            let host = host.clone();
                            move |_event, _window, cx| {
                                host.update(cx, |this, cx| {
                                    this.state.theme_choice = "dark";
                                    cx.notify();
                                });
                            }
                        }),
                ),
        )
        .child(
            Stepper::new(
                "foundation-stepper",
                format!("{}%", state.contrast.round() as i32),
            )
            .on_decrement({
                let host = host.clone();
                move |_event, _window, cx| {
                    host.update(cx, |this, cx| {
                        this.state.contrast = (this.state.contrast - 5.0).max(0.0);
                        cx.notify();
                    });
                }
            })
            .on_increment({
                let host = host.clone();
                move |_event, _window, cx| {
                    host.update(cx, |this, cx| {
                        this.state.contrast = (this.state.contrast + 5.0).min(100.0);
                        cx.notify();
                    });
                }
            })
            .on_reset({
                let host = host.clone();
                move |_event, _window, cx| {
                    host.update(cx, |this, cx| {
                        this.state.contrast = 60.0;
                        cx.notify();
                    });
                }
            }),
        )
}

fn nav_rows_sample() -> impl IntoElement {
    div()
        .w(gpui::px(280.0))
        .flex()
        .flex_col()
        .gap_1()
        .child(
            NavRow::new("nav-tasks", IconName::ListChecks, "Tasks")
                .count(3)
                .selected(true),
        )
        .child(NavRow::new(
            "nav-terminals",
            IconName::Terminal,
            "Terminals",
        ))
        .child(NavRow::new("nav-search", IconName::Search, "Search"))
}

fn tree_rows_sample() -> impl IntoElement {
    div()
        .w(gpui::px(300.0))
        .flex()
        .flex_col()
        .child(
            TreeRow::new("tr-1", IconName::Folder, "crates")
                .expandable(true)
                .depth(0),
        )
        .child(
            TreeRow::new("tr-2", IconName::Folder, "relay_ui_kit")
                .expandable(false)
                .depth(1),
        )
        .child(
            TreeRow::new("tr-3", IconName::FileText, "theme.rs")
                .depth(2)
                .selected(true),
        )
        .child(TreeRow::new("tr-4", IconName::FileText, "icon.rs").depth(2))
}

fn task_rows_sample() -> impl IntoElement {
    div()
        .w(gpui::px(320.0))
        .flex()
        .flex_col()
        .gap_1()
        .child(
            TaskRow::new(
                "task-1",
                TaskRowData {
                    title: "Wire diff pane".into(),
                    status_label: "RUNNING".into(),
                    status_tone: Tone::Accent,
                    branch: Some("relay/diff-pane".into()),
                    changed: 12,
                    review: 0,
                },
            )
            .selected(true),
        )
        .child(TaskRow::new(
            "task-2",
            TaskRowData {
                title: "Refactor terminal session".into(),
                status_label: "WAITING".into(),
                status_tone: Tone::Warning,
                branch: Some("relay/term".into()),
                changed: 3,
                review: 2,
            },
        ))
}

fn list_foundation_samples(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    theme: Theme,
) -> impl IntoElement {
    let selected = state.viewer_tab;
    let select_tree = {
        let host = host.clone();
        move |key: &'static str, _window: &mut gpui::Window, cx: &mut gpui::App| {
            host.update(cx, |this, cx| {
                this.state.viewer_tab = key;
                cx.notify();
            });
        }
    };

    div()
        .flex()
        .items_start()
        .gap_4()
        .flex_wrap()
        .child(
            div().w(gpui::px(320.0)).child(
                TreeView::new(
                    "foundation-tree-view",
                    vec![
                        TreeNode::new("tree:src", IconName::Folder, "src").expanded(true),
                        TreeNode::new("tree:components", IconName::Folder, "components")
                            .depth(1)
                            .expanded(true),
                        TreeNode::new("tree:list", IconName::Folder, "list")
                            .depth(2)
                            .expanded(true),
                        TreeNode::new("tree:item", IconName::FileText, "item.rs")
                            .depth(3)
                            .selected(selected == "tree:item"),
                    ],
                )
                .on_select(select_tree),
            ),
        )
        .child(div().w(gpui::px(340.0)).child(SectionedList::new(
            "foundation-sectioned-list",
            vec![
                    SectionedListGroup::new("Recent")
                        .count(2)
                        .child(
                            ListItem::new("recent-terminal")
                                .selected(selected == "recent:terminal")
                                .start_slot(Icon::new(IconName::Terminal))
                                .child(list_item_text(
                                    "Terminal surface",
                                    "PTY host shell preview",
                                    theme,
                                )),
                        )
                        .child(
                            ListItem::new("recent-diff")
                                .selected(selected == "recent:diff")
                                .start_slot(Icon::new(IconName::FileDiff))
                                .child(list_item_text("Diff viewer", "Unified file delta", theme)),
                        ),
                    SectionedListGroup::new("Pinned").child(
                        ListItem::new("pinned-command")
                            .start_slot(Icon::new(IconName::Zap))
                            .child(list_item_text(
                                "Command palette",
                                "Keyboard-first launcher",
                                theme,
                            )),
                    ),
                ],
        )))
}

fn list_item_text(title: &'static str, detail: &'static str, theme: Theme) -> impl IntoElement {
    div()
        .min_w_0()
        .flex()
        .flex_col()
        .child(
            div()
                .truncate()
                .text_sm()
                .text_color(theme.text)
                .child(title),
        )
        .child(
            div()
                .truncate()
                .text_size(gpui::px(11.0))
                .text_color(theme.text_muted)
                .child(detail),
        )
}

fn tab_samples(state: &GalleryState, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            Tabs::new(
                "demo-tabs",
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
                        cx.notify();
                    });
                }
            }),
        )
        .child(
            strip().child(
                SegmentedControl::new(
                    "seg-demo",
                    vec![
                        Segment::new("files", "Files"),
                        Segment::new("diff", "Diff"),
                        Segment::new("review", "Review"),
                    ],
                )
                .active(state.seg_tab)
                .on_select({
                    let host = host.clone();
                    move |key, _window, cx| {
                        host.update(cx, |this, cx| {
                            this.state.seg_tab = key;
                            cx.notify();
                        });
                    }
                }),
            ),
        )
        .child(Divider::horizontal())
        .child(
            EmptyState::new("No tasks yet", "Create a task to launch an agent.")
                .icon(IconName::ListChecks),
        )
}
