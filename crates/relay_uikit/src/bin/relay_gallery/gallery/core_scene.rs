use gpui::{Context, Entity, IntoElement, ParentElement, Styled, div, prelude::FluentBuilder};
use relay::Binding;
use relay_uikit::patterns::navigation::{Tab, Tabs};
use relay_uikit::patterns::overlay::{DropdownMenu, MenuItem};
use relay_uikit::workbench::{TaskRow, TaskRowData};
use relay_uikit::{
    Badge, Button, ButtonVariant, ColorField, ColorSwatch, CountBadge, Disclosure, Divider,
    EmptyState, FieldDescription, FieldLabel, FilterBar, FilterChip, Icon, IconButton, IconName,
    Label, LabelSize, ListItem, NavRow, Radio, SearchField, SectionedList, SectionedListGroup,
    Segment, SegmentedControl, Stepper, Theme, Tone, ToolbarGroup, TreeRow, TreeView,
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
    let contrast = state.contrast.get(cx);

    let stepper_body = stepper_filter_samples(state, contrast, cx);
    let input_body = input_choice_samples(state, host, theme);
    let label_body = label_badge_samples(state, cx);
    let list_body = list_core_samples(state, theme, cx);

    scene_stack()
        .child(section(cx, "Buttons", button_samples(state, host)))
        .child(section(cx, "Icon buttons", icon_button_samples(state, host)))
        .child(section(cx, "Steppers and filters", stepper_body))
        .child(section(cx, "Inputs and choices", input_body))
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
        .child(section(cx, "Labels, badges, disclosure", label_body))
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
        .child(section(cx, "Core list patterns", list_body))
        .child(section(cx, "Tabs and empty state", tab_samples(state)))
}

fn label_badge_samples(
    state: &GalleryState,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
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
                    Disclosure::bound(
                        "core-disclosure",
                        "Recent terminal sessions",
                        state.core_disclosure_open.clone(),
                    )
                    .detail("host-owned state")
                    .count(3),
                )
                .when(state.core_disclosure_open.get(cx), |this| {
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
                    SearchField::bound(
                        "core-search-field",
                        state.search_focus.clone(),
                        state.search_input.clone(),
                    )
                    .placeholder("Filter sessions"),
                ),
        )
        .child(
            div()
                .w(gpui::px(260.0))
                .flex()
                .flex_col()
                .gap_2()
                .child(FieldLabel::new("Radio"))
                .child(Radio::bound(
                    "core-radio-system",
                    state.radio_choice.clone(),
                    "system",
                    "System",
                ))
                .child(Radio::bound(
                    "core-radio-light",
                    state.radio_choice.clone(),
                    "light",
                    "Light",
                ))
                .child(Radio::bound(
                    "core-radio-dark",
                    state.radio_choice.clone(),
                    "dark",
                    "Dark",
                )),
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

fn button_samples(state: &GalleryState, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    strip()
        .child(
            Button::new("btn-primary", "Launch Agent")
                .primary()
                .icon(IconName::Play)
                .on_click({
                    let terminal_session = state.terminal_session.clone();
                    move |_event, _window, cx| {
                        terminal_session.set(cx, "codex");
                    }
                }),
        )
        .child(
            Button::new("btn-secondary", "Refresh")
                .icon(IconName::RefreshCw)
                .on_click({
                    let search_input = state.search_input.clone();
                    move |_event, _window, cx| {
                        search_input.update(cx, |s| {
                            s.clear();
                            true
                        });
                    }
                }),
        )
        .child(
            Button::new("btn-ghost", "Archive")
                .ghost()
                .icon(IconName::Archive)
                .on_click({
                    let auto_archive = state.auto_archive.clone();
                    move |_event, _window, cx| {
                        let current = auto_archive.get(cx);
                        auto_archive.set(cx, !current);
                    }
                }),
        )
        .child(
            Button::new("btn-disabled", "Disabled")
                .variant(ButtonVariant::Secondary)
                .disabled(true),
        )
}

fn icon_button_samples(state: &GalleryState, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    strip()
        .child(
            ToolbarGroup::new("core-toolbar-group")
                .child(
                    IconButton::new("ib-filter", IconName::ListFilter).on_click({
                        let seg_tab = state.seg_tab.clone();
                        move |_event, _window, cx| {
                            seg_tab.set(cx, "files");
                        }
                    }),
                )
                .child(
                    IconButton::new("ib-refresh", IconName::RefreshCw).on_click({
                        let search_input = state.search_input.clone();
                        move |_event, _window, cx| {
                            search_input.update(cx, |s| {
                                s.clear();
                                true
                            });
                        }
                    }),
                )
                .child(
                    IconButton::new("ib-settings", IconName::Settings).on_click({
                        let launcher_choice = state.launcher_choice.clone();
                        move |_event, _window, cx| {
                            launcher_choice.set(cx, "settings");
                        }
                    }),
                ),
        )
        .child(IconButton::new("ib-active", IconName::PanelLeft).active(true))
}

fn stepper_filter_samples(
    state: &GalleryState,
    contrast: f32,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            FilterBar::new("core-filter-bar")
                .child(session_filter_menu(state, cx))
                .child(
                    FilterChip::new("filter-running", "Running")
                        .icon(IconName::CircleDot)
                        .selected(state.filter_choice.get(cx) == "running")
                        .on_click({
                            let filter_choice = state.filter_choice.clone();
                            let filter_menu_open = state.filter_menu_open.clone();
                            move |_event, _window, cx| {
                                filter_choice.set(cx, "running");
                                filter_menu_open.set(cx, "");
                            }
                        }),
                )
                .child(project_filter_menu(state, cx)),
        )
        .child(
            Stepper::new("core-stepper", format!("{}%", contrast.round() as i32))
                .on_decrement({
                    let contrast = state.contrast.clone();
                    move |_event, _window, cx| {
                        let value = (contrast.get(cx) - 5.0).max(0.0);
                        contrast.set(cx, value);
                    }
                })
                .on_increment({
                    let contrast = state.contrast.clone();
                    move |_event, _window, cx| {
                        let value = (contrast.get(cx) + 5.0).min(100.0);
                        contrast.set(cx, value);
                    }
                })
                .on_reset({
                    let contrast = state.contrast.clone();
                    move |_event, _window, cx| {
                        contrast.set(cx, 60.0);
                    }
                }),
        )
}

fn session_filter_menu(
    state: &GalleryState,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
    let open = state.filter_menu_open.get(cx) == "sessions";
    let current_choice = state.filter_choice.get(cx);
    DropdownMenu::new(
        "core-session-filter-menu",
        FilterChip::new(
            "filter-all-sessions",
            match current_choice {
                "running" => "Running",
                "failed" => "Failed",
                _ => "All sessions",
            },
        )
        .icon(IconName::ListFilter)
        .count(4)
        .selected(current_choice != "all")
        .open(open)
        .dropdown(true)
        .on_click(toggle_filter_menu(state.filter_menu_open.clone(), "sessions", open)),
        vec![
            filter_action(
                state.filter_choice.clone(),
                state.filter_menu_open.clone(),
                "all",
                "All sessions",
                IconName::ListFilter,
            ),
            filter_action(
                state.filter_choice.clone(),
                state.filter_menu_open.clone(),
                "running",
                "Running",
                IconName::CircleDot,
            ),
            filter_action(
                state.filter_choice.clone(),
                state.filter_menu_open.clone(),
                "failed",
                "Failed",
                IconName::Archive,
            ),
        ],
    )
    .open(open)
    .min_width(190.0)
    .on_dismiss(close_filter_menu(state.filter_menu_open.clone()))
}

fn project_filter_menu(
    state: &GalleryState,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
    let open = state.filter_menu_open.get(cx) == "projects";
    let current_choice = state.project_filter_choice.get(cx);
    DropdownMenu::new(
        "core-project-filter-menu",
        FilterChip::new(
            "filter-projects",
            match current_choice {
                "relay" => "Relay",
                "gallery" => "Gallery",
                _ => "All projects",
            },
        )
        .icon(IconName::Folder)
        .selected(current_choice != "all-projects")
        .open(open)
        .dropdown(true)
        .on_click(toggle_filter_menu(
            state.filter_menu_open.clone(),
            "projects",
            open,
        )),
        vec![
            project_filter_action(
                state.project_filter_choice.clone(),
                state.filter_menu_open.clone(),
                "all-projects",
                "All projects",
            ),
            project_filter_action(
                state.project_filter_choice.clone(),
                state.filter_menu_open.clone(),
                "relay",
                "Relay",
            ),
            project_filter_action(
                state.project_filter_choice.clone(),
                state.filter_menu_open.clone(),
                "gallery",
                "Gallery",
            ),
        ],
    )
    .open(open)
    .min_width(180.0)
    .on_dismiss(close_filter_menu(state.filter_menu_open.clone()))
}

fn toggle_filter_menu(
    filter_menu_open: Binding<&'static str>,
    menu: &'static str,
    open: bool,
) -> impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static {
    move |_event, _window, cx| {
        filter_menu_open.set(cx, if open { "" } else { menu });
    }
}

fn close_filter_menu(
    filter_menu_open: Binding<&'static str>,
) -> impl Fn(&mut gpui::Window, &mut gpui::App) + 'static {
    move |_window, cx| {
        filter_menu_open.set(cx, "");
    }
}

fn filter_action(
    filter_choice: Binding<&'static str>,
    filter_menu_open: Binding<&'static str>,
    key: &'static str,
    label: &'static str,
    icon: IconName,
) -> MenuItem {
    MenuItem::new(label).icon(icon).on_click(move |_event, _window, cx| {
        filter_choice.set(cx, key);
        filter_menu_open.set(cx, "");
    })
}

fn project_filter_action(
    project_filter_choice: Binding<&'static str>,
    filter_menu_open: Binding<&'static str>,
    key: &'static str,
    label: &'static str,
) -> MenuItem {
    MenuItem::new(label)
        .icon(IconName::Folder)
        .on_click(move |_event, _window, cx| {
            project_filter_choice.set(cx, key);
            filter_menu_open.set(cx, "");
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
            TreeRow::new("tr-2", IconName::Folder, "relay_uikit::core")
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
    theme: Theme,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement + use<> {
    let selected = state.viewer_tab.get(cx);
    let tree_nodes = state.core_tree_nodes.get(cx);
    let select_tree = {
        let viewer_tab = state.viewer_tab.clone();
        move |key: &'static str, _window: &mut gpui::Window, cx: &mut gpui::App| {
            viewer_tab.set(cx, key);
        }
    };
    let toggle_tree = {
        let core_tree_src_open = state.core_tree_src_open.clone();
        let core_tree_components_open = state.core_tree_components_open.clone();
        let core_tree_list_open = state.core_tree_list_open.clone();
        move |key: &'static str, _window: &mut gpui::Window, cx: &mut gpui::App| {
            match key {
                "tree:src" => {
                    let current = core_tree_src_open.get(cx);
                    core_tree_src_open.set(cx, !current);
                }
                "tree:components" => {
                    let current = core_tree_components_open.get(cx);
                    core_tree_components_open.set(cx, !current);
                }
                "tree:list" => {
                    let current = core_tree_list_open.get(cx);
                    core_tree_list_open.set(cx, !current);
                }
                _ => {}
            }
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
                            .on_click(select_core_item(
                                state.viewer_tab.clone(),
                                "recent:terminal",
                            ))
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
                            .on_click(select_core_item(state.viewer_tab.clone(), "recent:diff"))
                            .child(list_item_text("Diff viewer", "Unified file delta", theme)),
                    ),
                SectionedListGroup::new("Pinned").child(
                    ListItem::new("pinned-command")
                        .height(gpui::px(48.0))
                        .start_slot(Icon::new(IconName::Zap))
                        .selected(selected == "pinned:command")
                        .on_click(select_core_item(
                            state.viewer_tab.clone(),
                            "pinned:command",
                        ))
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
    viewer_tab: Binding<&'static str>,
    key: &'static str,
) -> impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static {
    move |_event, _window, cx| {
        viewer_tab.set(cx, key);
    }
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

fn tab_samples(state: &GalleryState) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(Tabs::bound(
            "demo-tabs",
            vec![
                Tab::new("files", "Files").icon(IconName::FileText),
                Tab::new("diff", "Diff").icon(IconName::FileDiff).count(12),
                Tab::new("review", "Review")
                    .icon(IconName::MessageSquareText)
                    .count(3),
            ],
            state.seg_tab.clone(),
        ))
        .child(
            strip().child(SegmentedControl::bound(
                "seg-demo",
                vec![
                    Segment::new("files", "Files"),
                    Segment::new("diff", "Diff"),
                    Segment::new("review", "Review"),
                ],
                state.seg_tab.clone(),
            )),
        )
        .child(Divider::horizontal())
        .child(
            EmptyState::new("No tasks yet", "Create a task to launch an agent.")
                .icon(IconName::ListChecks),
        )
}
