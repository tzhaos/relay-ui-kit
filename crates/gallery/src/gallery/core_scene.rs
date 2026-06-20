use gpui::{Context, Entity, IntoElement, ParentElement, Styled, div, prelude::FluentBuilder};
use relay_ui_core::{
    Badge, Button, ButtonVariant, ColorField, ColorSwatch, CountBadge, Disclosure, Divider,
    EmptyState, FieldDescription, FieldLabel, FilterBar, FilterChip, Icon, IconButton, IconName,
    Label, LabelSize, ListItem, NavRow, Radio, SearchField, SectionedList, SectionedListGroup,
    Segment, SegmentedControl, Stepper, Theme, Tone, ToolbarGroup, TreeNode, TreeRow, TreeView,
};
use relay_ui_patterns::navigation::{Tab, Tabs};
use relay_ui_patterns::overlay::{DropdownMenu, MenuItem};
use relay_workbench::{TaskRow, TaskRowData};

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
            "Inputs and choices",
            input_choice_samples(state, host, theme),
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
            "Core list patterns",
            list_core_samples(state, host, theme),
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
                        "core-disclosure",
                        "Recent terminal sessions",
                        state.core_disclosure_open,
                    )
                    .detail("host-owned state")
                    .count(3)
                    .on_toggle({
                        let host = host.clone();
                        move |_event, _window, cx| {
                            host.update(cx, |this, cx| {
                                this.state.core_disclosure_open = !this.state.core_disclosure_open;
                                cx.notify();
                            });
                        }
                    }),
                )
                .when(state.core_disclosure_open, |this| {
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

fn input_choice_samples(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    theme: Theme,
) -> impl IntoElement {
    div()
        .flex()
        .items_start()
        .gap_4()
        .flex_wrap()
        .child(
            div()
                .w(gpui::px(320.0))
                .flex()
                .flex_col()
                .gap_2()
                .child(FieldLabel::new("SearchField"))
                .child(FieldDescription::new(
                    "Focusable input shell with host-owned text",
                ))
                .child(
                    SearchField::new("core-search-field", state.search_focus.clone())
                        .value(state.search_input.value())
                        .placeholder("Filter sessions")
                        .on_key({
                            let host = host.clone();
                            move |event, _window, cx| {
                                let mut handled = false;
                                host.update(cx, |this, cx| {
                                    let action = this.state.search_input.handle_key(event);
                                    if action.should_notify() {
                                        handled = true;
                                        cx.notify();
                                    }
                                });
                                handled
                            }
                        }),
                ),
        )
        .child(
            div()
                .w(gpui::px(260.0))
                .flex()
                .flex_col()
                .gap_2()
                .child(FieldLabel::new("Radio"))
                .child(
                    Radio::new(
                        "core-radio-system",
                        state.radio_choice == "system",
                        "System",
                    )
                    .on_click(set_radio_choice(host, "system")),
                )
                .child(
                    Radio::new("core-radio-light", state.radio_choice == "light", "Light")
                        .on_click(set_radio_choice(host, "light")),
                )
                .child(
                    Radio::new("core-radio-dark", state.radio_choice == "dark", "Dark")
                        .on_click(set_radio_choice(host, "dark")),
                ),
        )
        .child(
            div()
                .w(gpui::px(260.0))
                .flex()
                .flex_col()
                .gap_2()
                .child(FieldLabel::new("Color primitives"))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(ColorSwatch::new("core-swatch-accent", theme.accent))
                        .child(ColorSwatch::new("core-swatch-warning", theme.warning))
                        .child(ColorSwatch::new("core-swatch-danger", theme.danger)),
                )
                .child(ColorField::new("core-color-field", theme.accent, "#16A34A")),
        )
}

fn set_radio_choice(
    host: &Entity<GalleryScenesApp>,
    key: &'static str,
) -> impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static {
    let host = host.clone();
    move |_event, _window, cx| {
        host.update(cx, |this, cx| {
            this.state.radio_choice = key;
            cx.notify();
        });
    }
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
            ToolbarGroup::new("core-toolbar-group")
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
            FilterBar::new("core-filter-bar")
                .child(session_filter_menu(state, host))
                .child(
                    FilterChip::new("filter-running", "Running")
                        .icon(IconName::CircleDot)
                        .selected(state.filter_choice == "running")
                        .on_click({
                            let host = host.clone();
                            move |_event, _window, cx| {
                                host.update(cx, |this, cx| {
                                    this.state.filter_choice = "running";
                                    this.state.filter_menu_open = "";
                                    cx.notify();
                                });
                            }
                        }),
                )
                .child(project_filter_menu(state, host)),
        )
        .child(
            Stepper::new(
                "core-stepper",
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

fn session_filter_menu(state: &GalleryState, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    let open = state.filter_menu_open == "sessions";
    DropdownMenu::new(
        "core-session-filter-menu",
        FilterChip::new(
            "filter-all-sessions",
            match state.filter_choice {
                "running" => "Running",
                "failed" => "Failed",
                _ => "All sessions",
            },
        )
        .icon(IconName::ListFilter)
        .count(4)
        .selected(state.filter_choice != "all")
        .open(open)
        .dropdown(true)
        .on_click(toggle_filter_menu(host, "sessions", open)),
        vec![
            filter_action(host, "all", "All sessions", IconName::ListFilter),
            filter_action(host, "running", "Running", IconName::CircleDot),
            filter_action(host, "failed", "Failed", IconName::Archive),
        ],
    )
    .open(open)
    .min_width(190.0)
    .offset(0.0, 34.0)
    .on_dismiss(close_filter_menu(host))
}

fn project_filter_menu(state: &GalleryState, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    let open = state.filter_menu_open == "projects";
    DropdownMenu::new(
        "core-project-filter-menu",
        FilterChip::new(
            "filter-projects",
            match state.project_filter_choice {
                "relay" => "Relay",
                "gallery" => "Gallery",
                _ => "All projects",
            },
        )
        .icon(IconName::Folder)
        .selected(state.project_filter_choice != "all-projects")
        .open(open)
        .dropdown(true)
        .on_click(toggle_filter_menu(host, "projects", open)),
        vec![
            project_filter_action(host, "all-projects", "All projects"),
            project_filter_action(host, "relay", "Relay"),
            project_filter_action(host, "gallery", "Gallery"),
        ],
    )
    .open(open)
    .min_width(180.0)
    .offset(0.0, 34.0)
    .on_dismiss(close_filter_menu(host))
}

fn toggle_filter_menu(
    host: &Entity<GalleryScenesApp>,
    menu: &'static str,
    open: bool,
) -> impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static {
    let host = host.clone();
    move |_event, _window, cx| {
        host.update(cx, |this, cx| {
            this.state.filter_menu_open = if open { "" } else { menu };
            cx.notify();
        });
    }
}

fn close_filter_menu(
    host: &Entity<GalleryScenesApp>,
) -> impl Fn(&mut gpui::Window, &mut gpui::App) + 'static {
    let host = host.clone();
    move |_window, cx| {
        host.update(cx, |this, cx| {
            this.state.filter_menu_open = "";
            cx.notify();
        });
    }
}

fn filter_action(
    host: &Entity<GalleryScenesApp>,
    key: &'static str,
    label: &'static str,
    icon: IconName,
) -> MenuItem {
    MenuItem::new(label).icon(icon).on_click({
        let host = host.clone();
        move |_event, _window, cx| {
            host.update(cx, |this, cx| {
                this.state.filter_choice = key;
                this.state.filter_menu_open = "";
                cx.notify();
            });
        }
    })
}

fn project_filter_action(
    host: &Entity<GalleryScenesApp>,
    key: &'static str,
    label: &'static str,
) -> MenuItem {
    MenuItem::new(label).icon(IconName::Folder).on_click({
        let host = host.clone();
        move |_event, _window, cx| {
            host.update(cx, |this, cx| {
                this.state.project_filter_choice = key;
                this.state.filter_menu_open = "";
                cx.notify();
            });
        }
    })
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
            TreeRow::new("tr-2", IconName::Folder, "relay_ui_core")
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

fn list_core_samples(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    theme: Theme,
) -> impl IntoElement {
    let selected = state.viewer_tab;
    let tree_nodes = core_tree_nodes(state);
    let select_tree = {
        let host = host.clone();
        move |key: &'static str, _window: &mut gpui::Window, cx: &mut gpui::App| {
            host.update(cx, |this, cx| {
                this.state.viewer_tab = key;
                cx.notify();
            });
        }
    };
    let toggle_tree = {
        let host = host.clone();
        move |key: &'static str, _window: &mut gpui::Window, cx: &mut gpui::App| {
            host.update(cx, |this, cx| {
                match key {
                    "tree:src" => {
                        this.state.core_tree_src_open = !this.state.core_tree_src_open;
                    }
                    "tree:components" => {
                        this.state.core_tree_components_open =
                            !this.state.core_tree_components_open;
                    }
                    "tree:list" => {
                        this.state.core_tree_list_open = !this.state.core_tree_list_open;
                    }
                    _ => {}
                }
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
                TreeView::new("core-tree-view", tree_nodes)
                    .on_select(select_tree)
                    .on_toggle(toggle_tree),
            ),
        )
        .child(div().w(gpui::px(340.0)).child(SectionedList::new(
            "core-sectioned-list",
            vec![
                    SectionedListGroup::new("Recent")
                        .count(2)
                        .child(
                            ListItem::new("recent-terminal")
                                .height(gpui::px(48.0))
                                .selected(selected == "recent:terminal")
                                .start_slot(Icon::new(IconName::Terminal))
                                .on_click(select_core_item(host, "recent:terminal"))
                                .child(list_item_text(
                                    "Terminal surface",
                                    "PTY host shell preview",
                                    theme,
                                )),
                        )
                        .child(
                            ListItem::new("recent-diff")
                                .height(gpui::px(48.0))
                                .selected(selected == "recent:diff")
                                .start_slot(Icon::new(IconName::FileDiff))
                                .on_click(select_core_item(host, "recent:diff"))
                                .child(list_item_text("Diff viewer", "Unified file delta", theme)),
                        ),
                    SectionedListGroup::new("Pinned").child(
                        ListItem::new("pinned-command")
                            .height(gpui::px(48.0))
                            .start_slot(Icon::new(IconName::Zap))
                            .selected(selected == "pinned:command")
                            .on_click(select_core_item(host, "pinned:command"))
                            .child(list_item_text(
                                "Command palette",
                                "Keyboard-first launcher",
                                theme,
                            )),
                    ),
                ],
        )))
}

fn select_core_item(
    host: &Entity<GalleryScenesApp>,
    key: &'static str,
) -> impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static {
    let host = host.clone();
    move |_event, _window, cx| {
        host.update(cx, |this, cx| {
            this.state.viewer_tab = key;
            cx.notify();
        });
    }
}

fn core_tree_nodes(state: &GalleryState) -> Vec<TreeNode> {
    let mut nodes =
        vec![TreeNode::new("tree:src", IconName::Folder, "src").expanded(state.core_tree_src_open)];

    if state.core_tree_src_open {
        nodes.push(
            TreeNode::new("tree:components", IconName::Folder, "components")
                .depth(1)
                .expanded(state.core_tree_components_open),
        );
    }

    if state.core_tree_src_open && state.core_tree_components_open {
        nodes.push(
            TreeNode::new("tree:list", IconName::Folder, "list")
                .depth(2)
                .expanded(state.core_tree_list_open),
        );
    }

    if state.core_tree_src_open && state.core_tree_components_open && state.core_tree_list_open {
        nodes.push(
            TreeNode::new("tree:item", IconName::FileText, "item.rs")
                .depth(3)
                .selected(state.viewer_tab == "tree:item"),
        );
    }

    nodes
}

fn list_item_text(title: &'static str, detail: &'static str, theme: Theme) -> impl IntoElement {
    div()
        .min_w_0()
        .flex_1()
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
