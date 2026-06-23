//! Core kit gallery — every component wired to relay signals with toast feedback.

use gpui::{
    App, Context, Entity, IntoElement, ParentElement, Role, Styled, div, prelude::FluentBuilder, px,
};
use relay_uikit::{
    Button, ButtonVariant, Checkbox, Disclosure, Divider, FilterBar, FilterChip, Icon, IconButton,
    IconName, IconSize, Label, LabelSize, ListItem, NavRow, NumberInput, Radio, SearchField,
    Segment, SegmentedControl, Slider, Stepper, TextInput, TextInputState, Theme, ThemePreviewKind,
    Toggle, TreeRow,
};

use super::GalleryScenesApp;
use super::shared::{scene_stack, section, strip};
use super::{GalleryContentTab, GalleryState};

pub(super) fn render(
    state: &GalleryState,
    host: &Entity<GalleryScenesApp>,
    window: &gpui::Window,
    _theme: Theme,
    cx: &mut Context<GalleryScenesApp>,
) -> impl IntoElement {
    let disclosure_open = state.core_disclosure_open.get(cx);
    let name_focused = state.name_focus.is_focused(window);

    scene_stack()
        .child(section(cx, "Buttons", button_sample(host)))
        .child(section(cx, "Icon Buttons", icon_button_sample(host)))
        .child(section(
            cx,
            "Toggle · Checkbox · Radio",
            choice_sample(state, host),
        ))
        .child(section(
            cx,
            "Text Input",
            text_input_sample(state, name_focused),
        ))
        .child(section(cx, "Search & Filter", search_sample(state, host)))
        .child(section(
            cx,
            "Number & Slider & Stepper",
            number_sample(state, host),
        ))
        .child(section(cx, "Segmented Control", segmented_sample(state)))
        .child(section(
            cx,
            "Disclosure",
            disclosure_sample(state, disclosure_open),
        ))
        .child(section(cx, "List & Tree Rows", tree_sample(state, cx)))
}

fn toast(
    host: Entity<GalleryScenesApp>,
    msg: impl Into<String>,
) -> impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut App) {
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
                TextInput::new(
                    "demo-dis",
                    state.name_focus.clone(),
                    &TextInputState::with_text("Disabled input"),
                )
                .disabled(true),
            ),
        )
}

fn search_sample(state: &GalleryState, host: &Entity<GalleryScenesApp>) -> impl IntoElement {
    let h = host.clone();
    let clear_host = h.clone();
    let search_input = state.search_input.clone();
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
                    search_input.update(cx, |state| {
                        state.clear();
                        true
                    });
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
                        move |v: i32, _: &mut gpui::Window, cx: &mut App| {
                            h.update(cx, |this, cx| {
                                this.add_feedback_toast(cx, format!("Font size: {v}px"))
                            });
                        }
                    }),
            ),
        )
        .child(strip().child(
            Slider::bound("demo-slider", state.contrast.clone(), 0.0, 100.0).on_change({
                let h = host.clone();
                move |v: f32, _: &mut gpui::Window, cx: &mut App| {
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
        .child(TreeRow::new("demo-tree", IconName::Folder, "src/components").expandable(true))
        .child(
            TreeRow::bound(
                "demo-tree-file",
                IconName::FileText,
                "button.rs",
                state.core_disclosure_open.clone(),
            )
            .depth(1)
            .expanded_bound(state.core_disclosure_open.clone()),
        )
        .when(state.core_disclosure_open.get(cx), |this| {
            this.child(TreeRow::new("demo-tree-nested", IconName::FileText, "mod.rs").depth(2))
        })
}
