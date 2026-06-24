//! Core kit gallery — every component wired to relay signals with toast feedback.

use gpui::{
    App, Context, Entity, IntoElement, ParentElement, Role, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay_uikit::interaction::{OpenState, SelectionBinding};
use relay_uikit::{
    ActiveTheme, Badge, Button, ButtonVariant, Checkbox, ColorField, ColorSwatch, CountBadge,
    Disclosure, Divider, FieldDescription, FieldLabel, FilterBar, FilterChip, ForEach, Icon,
    IconButton, IconName, IconSize, Label, LabelSize, ListItem, NavRow, NumberInput,
    NumberInputLayout, PanelHeader, Radio, SearchField, SectionedList, SectionedListGroup, Segment,
    SegmentedControl, Slider, StatusDot, Stepper, TextInput, Theme, ThemePreviewKind, Toggle, Tone,
    ToolbarGroup, TreeNode, TreeRow, TreeView,
};

use super::GalleryScenesApp;
use super::shared::{scene_stack, section, strip};
use super::{CoreTreeNodeKey, GalleryContentTab, GalleryState};

pub(super) const COVERAGE_TITLES: [&str; 12] = [
    "Buttons",
    "Icon Buttons",
    "Badges · Color · Chrome",
    "Toggle · Checkbox · Radio",
    "Text Input",
    "Field Anatomy & Reactive Lists",
    "Search & Filter",
    "Number & Slider & Stepper",
    "Segmented Control",
    "Disclosure",
    "Sectioned List & Tree View",
    "List & Tree Rows",
];

pub(super) fn render(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    window: &Window,
    _theme: Theme,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement {
    let disclosure_open = state.core_disclosure_open.get(cx);
    let name_focused = state.name_focus.is_focused(window);

    scene_stack()
        .child(section(cx, COVERAGE_TITLES[0], button_sample(host)))
        .child(section(cx, COVERAGE_TITLES[1], icon_button_sample(host)))
        .child(section(cx, COVERAGE_TITLES[2], chrome_sample(host)))
        .child(section(cx, COVERAGE_TITLES[3], choice_sample(state, host)))
        .child(section(
            cx,
            COVERAGE_TITLES[4],
            text_input_sample(state, name_focused),
        ))
        .child(section(
            cx,
            COVERAGE_TITLES[5],
            field_primitives_sample(state, host, window, cx),
        ))
        .child(section(cx, COVERAGE_TITLES[6], search_sample(state, host)))
        .child(section(cx, COVERAGE_TITLES[7], number_sample(state, host)))
        .child(section(cx, COVERAGE_TITLES[8], segmented_sample(state)))
        .child(section(
            cx,
            COVERAGE_TITLES[9],
            disclosure_sample(state, disclosure_open),
        ))
        .child(section(
            cx,
            COVERAGE_TITLES[10],
            structured_collection_sample(state, host, cx),
        ))
        .child(section(cx, COVERAGE_TITLES[11], tree_sample(state, cx)))
}

fn toast(
    host: Entity<GalleryScenesApp>,
    msg: impl Into<String>,
) -> impl Fn(&gpui::ClickEvent, &mut Window, &mut App) {
    let msg = msg.into();
    move |_, _, cx: &mut App| {
        host.update(cx, |this, cx| this.add_feedback_toast(cx, msg.clone()));
    }
}

// ── Button samples ────────────────────────────────────────────────────────

fn button_sample(host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    let h = host.clone();
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            strip()
                .child(
                    Button::new("btn-pri", "Primary")
                        .primary()
                        .on_click(toast(h.clone(), "Primary")),
                )
                .child(
                    Button::new("btn-sec", "Secondary")
                        .variant(ButtonVariant::Secondary)
                        .on_click(toast(h.clone(), "Secondary")),
                )
                .child(
                    Button::new("btn-ghost", "Ghost")
                        .ghost()
                        .on_click(toast(h.clone(), "Ghost")),
                )
                .child(
                    Button::new("btn-dang", "Danger")
                        .danger()
                        .on_click(toast(h.clone(), "Danger")),
                ),
        )
        .child(
            strip()
                .child(
                    Button::new("btn-pri-icon", "With Icon")
                        .primary()
                        .icon(IconName::Play)
                        .on_click(toast(h.clone(), "Play")),
                )
                .child(Button::new("btn-dis", "Disabled").primary().disabled(true)),
        )
        .child(
            strip()
                .child(
                    Button::new("btn-sec-icon", "Refresh")
                        .variant(ButtonVariant::Secondary)
                        .icon(IconName::RefreshCw)
                        .on_click(toast(h.clone(), "Refresh")),
                )
                .child(
                    Button::new("btn-ghost-icon", "Settings")
                        .ghost()
                        .icon(IconName::Settings)
                        .on_click(toast(h, "Settings")),
                ),
        )
}

fn icon_button_sample(host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    let h = host.clone();
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            strip()
                .child(
                    IconButton::new("ib-plus", IconName::Plus)
                        .size(IconSize::Small)
                        .aria_label("Add item")
                        .on_click(toast(h.clone(), "Plus")),
                )
                .child(
                    IconButton::new("ib-search", IconName::Search)
                        .size(IconSize::Small)
                        .aria_label("Search")
                        .on_click(toast(h.clone(), "Search")),
                )
                .child(
                    IconButton::new("ib-archive", IconName::Archive)
                        .size(IconSize::Small)
                        .aria_label("Archive")
                        .on_click(toast(h.clone(), "Archive")),
                )
                .child(
                    IconButton::new("ib-panel", IconName::PanelLeft)
                        .size(IconSize::Small)
                        .aria_label("Toggle left panel")
                        .active(true)
                        .on_click(toast(h, "Panel toggle")),
                ),
        )
        .child(
            strip().child(
                IconButton::new("ib-dis", IconName::Plus)
                    .size(IconSize::Small)
                    .disabled(true),
            ),
        )
}

fn chrome_sample(host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    let accent = gpui::rgb(0x16a34a).into();
    let h = host.clone();

    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            strip()
                .child(Badge::new("ACTIVE").tone(Tone::Accent).soft())
                .child(Badge::new("READONLY").tone(Tone::Secondary))
                .child(CountBadge::new(7).tone(Tone::Accent))
                .child(CountBadge::new(128).tone(Tone::Warning))
                .child(ColorSwatch::new("chrome-accent-swatch", accent))
                .child(ColorField::new("chrome-accent-field", accent, "#16A34A")),
        )
        .child(
            div()
                .max_w(px(560.0))
                .rounded(px(12.0))
                .border_1()
                .border_color(gpui::rgb(0x2a2f37))
                .bg(gpui::rgb(0x171b20))
                .overflow_hidden()
                .child(
                    PanelHeader::new("Terminal")
                        .icon(IconName::Terminal)
                        .trailing(
                            ToolbarGroup::new("core-panel-toolbar")
                                .child(
                                    IconButton::new("core-panel-search", IconName::Search)
                                        .size(IconSize::Small)
                                        .aria_label("Search output")
                                        .on_click(toast(h.clone(), "Panel search")),
                                )
                                .child(
                                    IconButton::new("core-panel-split", IconName::PanelLeft)
                                        .size(IconSize::Small)
                                        .aria_label("Split terminal")
                                        .on_click(toast(h.clone(), "Panel split")),
                                )
                                .child(
                                    IconButton::new("core-panel-more", IconName::Ellipsis)
                                        .size(IconSize::Small)
                                        .aria_label("Open panel actions")
                                        .on_click(toast(h, "Panel actions")),
                                ),
                        ),
                )
                .child(
                    div()
                        .p_3()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .text_color(gpui::rgb(0xd8dbe0))
                                .child("relay_uikit now shows its chrome primitives in a real pane shell."),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(gpui::rgb(0x8b93a1))
                                .child("PanelHeader and ToolbarGroup should feel production-ready when embedded, not only as isolated widgets."),
                        ),
                ),
        )
}

// ── Choice samples ────────────────────────────────────────────────────────

fn choice_sample(state: &GalleryState, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    let h = host.clone();
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            strip()
                .child(
                    Toggle::bound("demo-toggle", state.notifications.clone())
                        .label("Notifications")
                        .on_click(toast(h.clone(), "Toggle toggled")),
                )
                .child(
                    Checkbox::bound("demo-check", state.auto_archive.clone())
                        .label("Auto archive")
                        .on_click(toast(h.clone(), "Checkbox toggled")),
                ),
        )
        .child(
            strip()
                .child(
                    Radio::bound(
                        "demo-radio-sys",
                        state.radio_choice.clone(),
                        ThemePreviewKind::System,
                        "System",
                    )
                    .on_click(toast(h.clone(), "Radio: system")),
                )
                .child(
                    Radio::bound(
                        "demo-radio-light",
                        state.radio_choice.clone(),
                        ThemePreviewKind::Light,
                        "Light",
                    )
                    .on_click(toast(h.clone(), "Radio: light")),
                )
                .child(
                    Radio::bound(
                        "demo-radio-dark",
                        state.radio_choice.clone(),
                        ThemePreviewKind::Dark,
                        "Dark",
                    )
                    .on_click(toast(h, "Radio: dark")),
                ),
        )
}

// ── Text input samples ────────────────────────────────────────────────────

fn text_input_sample(state: &GalleryState, focused: bool) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div().max_w(px(320.0)).child(
                TextInput::bound(
                    "demo-name",
                    state.name_focus.clone(),
                    state.name_input.clone(),
                )
                .leading_icon(IconName::Search)
                .placeholder("Enter your name — type to see live text")
                .focused(focused),
            ),
        )
        .child(
            div().max_w(px(320.0)).child(
                TextInput::bound(
                    "demo-dis",
                    state.disabled_text_focus.clone(),
                    state.disabled_text_input.clone(),
                )
                .disabled(true),
            ),
        )
}

fn field_primitives_sample(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    window: &Window,
    cx: &App,
) -> impl IntoElement {
    let theme = *cx.theme();
    let branch_focused = state.core_branch_focus.is_focused(window);
    let quick_item_count = state.core_quick_items.read(cx, |items| items.len());
    let quick_items_empty = quick_item_count == 0;

    div()
        .grid()
        .grid_cols(2)
        .gap_4()
        .child(
            div()
                .max_w(px(360.0))
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(FieldLabel::new("Default branch"))
                        .child(FieldDescription::new(
                            "Low-level field primitives stay separate so product surfaces can place help text, validation, and inputs without the constraints of SettingsRow.",
                        ))
                        .child(
                            div().pt_1().child(
                                TextInput::bound(
                                    "core-branch-input",
                                    state.core_branch_focus.clone(),
                                    state.core_branch_input.clone(),
                                )
                                .placeholder("relay_v2")
                                .focused(branch_focused),
                            ),
                        ),
                )
                .child(
                    div()
                        .rounded(px(10.0))
                        .border_1()
                        .border_color(theme.border)
                        .bg(theme.inset)
                        .p_3()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(FieldLabel::new("Why this matters"))
                        .child(FieldDescription::new(
                            "These primitives are the escape hatch for dense multi-column forms, inline validation, and any layout where a single canned row is too limiting.",
                        )),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    strip()
                        .child(
                            Button::new("core-quick-rotate", "Rotate")
                                .ghost()
                                .icon(IconName::RefreshCw)
                                .on_click({
                                    let host = host.clone();
                                    move |_event, _window, cx| {
                                        host.update(cx, |this, cx| {
                                            this.rotate_core_quick_items(cx);
                                        });
                                    }
                                }),
                        )
                        .child(
                            Button::new("core-quick-add", "Add slice")
                                .ghost()
                                .icon(IconName::Plus)
                                .on_click({
                                    let host = host.clone();
                                    move |_event, _window, cx| {
                                        host.update(cx, |this, cx| {
                                            this.add_core_quick_item(cx);
                                        });
                                    }
                                }),
                        )
                        .child(
                            Button::new("core-quick-remove", "Remove first")
                                .ghost()
                                .icon(IconName::Archive)
                                .disabled(quick_items_empty)
                                .on_click({
                                    let host = host.clone();
                                    move |_event, _window, cx| {
                                        host.update(cx, |this, cx| {
                                            this.remove_core_quick_item(cx);
                                        });
                                    }
                                }),
                        ),
                )
                .child(
                    div()
                        .rounded(px(10.0))
                        .border_1()
                        .border_color(theme.border)
                        .bg(theme.chrome)
                        .p_2()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .when(quick_items_empty, |this| {
                            this.child(
                                div()
                                    .rounded(px(8.0))
                                    .bg(theme.panel)
                                    .p_3()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .child("No quick items remain. Add one to validate empty-state recovery."),
                            )
                        })
                        .child(
                            ForEach::new("core-quick-items", state.core_quick_items.clone())
                                .key(|item| item.id as usize)
                                .render_item(move |item, _window, _cx| {
                                    ListItem::new(format!("core-quick-item-{}", item.id))
                                        .start_slot(StatusDot::new(item.tone))
                                        .child(item.label.clone())
                                        .end_slot(
                                            Badge::new(core_quick_item_tone_label(item.tone))
                                                .tone(item.tone)
                                                .soft(),
                                        )
                                        .into_any_element()
                                }),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child(format!(
                            "ForEach is the cheap reactive path for product rows backed directly by a signal. {quick_item_count} item{} live.",
                            if quick_item_count == 1 { "" } else { "s" }
                        )),
                ),
        )
}

fn search_sample(state: &GalleryState, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    let h = host.clone();
    let clear_host = h.clone();
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div().max_w(px(280.0)).child(
                SearchField::bound(
                    "demo-search",
                    state.search_focus.clone(),
                    state.search_input.clone(),
                )
                .placeholder("Filter items...")
                .on_clear(move |_, _, cx| {
                    clear_host.update(cx, |this, cx| {
                        this.add_feedback_toast(cx, "Search cleared");
                    });
                }),
            ),
        )
        .child(
            strip().child(
                FilterBar::new("demo-fb")
                    .child(
                        FilterChip::new("chip-all", "All")
                            .icon(IconName::LayoutGrid)
                            .on_click(toast(h.clone(), "Filter: All")),
                    )
                    .child(
                        FilterChip::new("chip-run", "running")
                            .icon(IconName::Play)
                            .selected(true)
                            .on_click(toast(h, "Filter: running")),
                    )
                    .child(FilterChip::new("chip-ro", "readonly").disabled(true)),
            ),
        )
}

fn core_quick_item_tone_label(tone: Tone) -> &'static str {
    match tone {
        Tone::Accent => "Active",
        Tone::Info => "Info",
        Tone::Warning => "Review",
        Tone::Secondary => "Queued",
        Tone::Danger => "Blocked",
        Tone::Muted => "Muted",
    }
}

// ── Number / Slider / Stepper ─────────────────────────────────────────────

fn number_sample(state: &GalleryState, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            strip().child(
                NumberInput::bound("demo-num", state.ui_font_size.clone())
                    .input_bound(
                        state.ui_font_size_focus.clone(),
                        state.ui_font_size_input.clone(),
                    )
                    .range(10, 24)
                    .suffix("px")
                    .on_change({
                        let h = host.clone();
                        move |v: i32, _: &mut Window, cx: &mut App| {
                            h.update(cx, |this, cx| {
                                this.add_feedback_toast(cx, format!("Font size: {v}px"))
                            });
                        }
                    }),
            ),
        )
        .child(
            strip().child(
                NumberInput::bound("demo-num-trailing", state.ui_font_size.clone())
                    .layout(NumberInputLayout::ControlsTrailing)
                    .range(10, 24)
                    .suffix("px")
                    .on_change({
                        let h = host.clone();
                        move |v: i32, _: &mut Window, cx: &mut App| {
                            h.update(cx, |this, cx| {
                                this.add_feedback_toast(
                                    cx,
                                    format!("Trailing number layout: {v}px"),
                                )
                            });
                        }
                    }),
            ),
        )
        .child(strip().child(
            Slider::bound("demo-slider", state.contrast.clone(), 0.0, 100.0).on_change({
                let h = host.clone();
                move |v: f32, _: &mut Window, cx: &mut App| {
                    h.update(cx, |this, cx| {
                        this.add_feedback_toast(cx, format!("Contrast: {:.0}%", v))
                    });
                }
            }),
        ))
        .child(
            strip().child(
                Stepper::bound("demo-step", state.ui_font_size.clone())
                    .range(10, 24)
                    .on_decrement(toast(host.clone(), "Stepper: -1"))
                    .on_increment(toast(host.clone(), "Stepper: +1")),
            ),
        )
}

// ── Segmented control ─────────────────────────────────────────────────────

fn segmented_sample(state: &GalleryState) -> impl IntoElement {
    SegmentedControl::bound(
        "demo-seg",
        vec![
            Segment::new(GalleryContentTab::Files, "Files"),
            Segment::new(GalleryContentTab::Diff, "Diff"),
            Segment::new(GalleryContentTab::Review, "Review"),
        ],
        state.content_tab.clone(),
    )
}

// ── Disclosure ────────────────────────────────────────────────────────────

fn disclosure_sample(state: &GalleryState, open: bool) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            Disclosure::bound(
                "demo-disc",
                "Advanced settings",
                state.core_disclosure_open.clone(),
            )
            .detail("Click or press Space to expand"),
        )
        .when(open, |this| {
            this.child(
                div()
                    .pl_4()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        Label::new("These settings appear when the disclosure is open.")
                            .size(LabelSize::Small),
                    )
                    .child(
                        Checkbox::bound("demo-disc-opt", state.auto_archive.clone())
                            .label("Auto archive"),
                    ),
            )
        })
}

fn structured_collection_sample(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    cx: &App,
) -> impl IntoElement {
    let tree_selected = state.core_tree_selected.get(cx);

    div()
        .grid()
        .grid_cols(2)
        .gap_4()
        .child(sectioned_list_sample())
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    TreeView::new("core-tree-view", core_tree_nodes(state, cx))
                        .on_toggle({
                            let host = host.clone();
                            move |key, _window, cx| {
                                host.update(cx, |this, cx| match key {
                                    CoreTreeNodeKey::Src => {
                                        this.state.core_tree_src_open.update(cx, |open| {
                                            *open = !*open;
                                            true
                                        });
                                    }
                                    CoreTreeNodeKey::Components => {
                                        this.state.core_tree_components_open.update(cx, |open| {
                                            *open = !*open;
                                            true
                                        });
                                    }
                                    _ => {}
                                });
                            }
                        })
                        .on_select({
                            let host = host.clone();
                            move |key, _window, cx| {
                                host.update(cx, |this, cx| {
                                    this.state.core_tree_selected.set(cx, key);
                                    this.add_feedback_toast(
                                        cx,
                                        format!("Tree selected: {}", core_tree_label(key)),
                                    );
                                });
                            }
                        }),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().text_muted)
                        .child(format!("Selected file: {}", core_tree_label(tree_selected))),
                ),
        )
}

fn sectioned_list_sample() -> impl IntoElement {
    let pinned = SectionedListGroup::new("Pinned")
        .count(2)
        .trailing(CountBadge::new(2).tone(Tone::Accent))
        .child(
            ListItem::new("sectioned-pinned-session")
                .start_slot(Icon::new(IconName::Terminal).size(IconSize::Small))
                .end_slot(Badge::new("LIVE").tone(Tone::Accent).soft())
                .child("relay_v2 migration"),
        )
        .child(
            ListItem::new("sectioned-pinned-review")
                .start_slot(Icon::new(IconName::MessageSquareText).size(IconSize::Small))
                .end_slot(Label::new("3 comments").size(LabelSize::Small))
                .child("Product review queue"),
        );
    let queued = SectionedListGroup::new("Queued")
        .count(2)
        .child(
            ListItem::new("sectioned-queued-docs")
                .start_slot(Icon::new(IconName::FileText).size(IconSize::Small))
                .child("Rewrite crate docs"),
        )
        .child(
            ListItem::new("sectioned-queued-gallery")
                .start_slot(Icon::new(IconName::LayoutGrid).size(IconSize::Small))
                .child("Land missing gallery surfaces"),
        );

    SectionedList::new("core-sectioned-list", vec![pinned, queued])
}

fn core_tree_nodes(state: &GalleryState, cx: &App) -> Vec<TreeNode<CoreTreeNodeKey>> {
    let src_open = state.core_tree_src_open.get(cx);
    let components_open = state.core_tree_components_open.get(cx);
    let selected = state.core_tree_selected.get(cx);
    let mut nodes =
        vec![TreeNode::new(CoreTreeNodeKey::Src, IconName::Folder, "src").expanded(src_open)];

    if src_open {
        nodes.push(
            TreeNode::new(CoreTreeNodeKey::Components, IconName::Folder, "components")
                .depth(1)
                .expanded(components_open),
        );
    }

    if src_open && components_open {
        nodes.push(
            TreeNode::new(CoreTreeNodeKey::ButtonRs, IconName::FileText, "button.rs")
                .depth(2)
                .selected(selected == CoreTreeNodeKey::ButtonRs),
        );
        nodes.push(
            TreeNode::new(
                CoreTreeNodeKey::TextInputRs,
                IconName::FileText,
                "text_input.rs",
            )
            .depth(2)
            .selected(selected == CoreTreeNodeKey::TextInputRs),
        );
    }

    nodes
}

fn core_tree_label(key: CoreTreeNodeKey) -> &'static str {
    match key {
        CoreTreeNodeKey::Src => "src",
        CoreTreeNodeKey::Components => "components",
        CoreTreeNodeKey::ButtonRs => "button.rs",
        CoreTreeNodeKey::TextInputRs => "text_input.rs",
    }
}

// ── List & Tree rows ──────────────────────────────────────────────────────

fn tree_sample(state: &GalleryState, cx: &App) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            NavRow::new("demo-nav", IconName::Terminal, "Active session")
                .count(3)
                .selected(true),
        )
        .child(NavRow::new("demo-nav2", IconName::PanelLeft, "Files"))
        .child(Divider::horizontal())
        .child(
            ListItem::new("demo-list-selected")
                .role(Role::ListItem)
                .aria_label("Selected inbox thread")
                .selected(true)
                .start_slot(Icon::new(IconName::Archive).size(IconSize::Small))
                .end_slot(Label::new("Pinned").size(LabelSize::Small))
                .child("relay_v2 migration thread"),
        )
        .child(
            ListItem::new("demo-list-toggle")
                .role(Role::ListItem)
                .aria_label("Readonly source file")
                .toggled(true)
                .start_slot(Icon::new(IconName::FileText).size(IconSize::Small))
                .end_slot(Label::new("Readonly").size(LabelSize::Small))
                .child("src/components/button.rs"),
        )
        .child(
            TreeRow::new("demo-tree", IconName::Folder, "src/components")
                .selection_binding(SelectionBinding::binding(
                    state.core_tree_selected.clone(),
                    CoreTreeNodeKey::Components,
                ))
                .open_state(OpenState::binding(state.core_disclosure_open.clone()))
                .expandable(state.core_disclosure_open.get(cx)),
        )
        .child(
            TreeRow::new("demo-tree-file", IconName::FileText, "button.rs")
                .selection_binding(SelectionBinding::binding(
                    state.core_tree_selected.clone(),
                    CoreTreeNodeKey::ButtonRs,
                ))
                .depth(1),
        )
        .when(state.core_disclosure_open.get(cx), |this| {
            this.child(TreeRow::new("demo-tree-nested", IconName::FileText, "mod.rs").depth(2))
        })
}
