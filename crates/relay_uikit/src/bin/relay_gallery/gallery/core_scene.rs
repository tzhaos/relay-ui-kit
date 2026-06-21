//! Core kit gallery — every component wired to relay signals and actually working.

use gpui::{App, Entity, IntoElement, ParentElement, Styled, div, prelude::FluentBuilder, px};
use relay::Signal;
use relay_uikit::{
    ActiveTheme, Badge, Button, ButtonVariant, Checkbox, CountBadge, Disclosure, Divider,
    FilterBar, FilterChip, Icon, IconButton, IconName, IconSize, Label, LabelSize, ListItem,
    NavRow, NumberInput, Radio, SearchField, SegmentedControl, Segment, Slider, Stepper,
    TextInput, TextInputState, Theme, Toggle, TreeRow,
};
use relay_uikit::patterns::navigation::{Tab, Tabs};

use super::GalleryScenesApp;
use super::GalleryState;
use super::shared::{scene_stack, section, strip};

pub(super) fn render(
    state: &GalleryState,
    _host: &Entity<GalleryScenesApp>,
    window: &gpui::Window,
    theme: Theme,
    cx: &mut gpui::Context<GalleryScenesApp>,
) -> impl IntoElement {
    let event_text = state.overlay_event.get(cx);
    let disclosure_open = state.core_disclosure_open.get(cx);
    let name_focused = state.name_focus.is_focused(window);

    let buttons = section(cx, "Buttons", button_sample(state));
    let icons = section(cx, "Icon Buttons", icon_button_sample(state));
    let choices = section(cx, "Choices — Toggle, Checkbox, Radio", choice_sample(state));
    let inputs = section(cx, "Text Input", text_input_sample(state, name_focused));
    let search = section(cx, "Search & Filter", search_sample(state));
    let numbers = section(cx, "Number & Slider & Stepper", number_sample(state));
    let seg = section(cx, "Segmented Control", segmented_sample(state));
    let disc = section(cx, "Disclosure", disclosure_sample(state, disclosure_open));
    let lists = section(cx, "List & Tree Rows", list_sample(theme));

    scene_stack()
        .child(event_header(theme, &event_text))
        .child(buttons).child(icons).child(choices)
        .child(inputs).child(search).child(numbers)
        .child(seg).child(disc).child(lists)
}

fn event_header(theme: Theme, text: &str) -> impl IntoElement {
    div()
        .px_3().py_2().rounded(px(relay_uikit::radius::MD))
        .bg(theme.panel).border_1().border_color(theme.border)
        .flex().items_center().gap_2()
        .child(Label::new("Last event:").size(LabelSize::XSmall))
        .child(div().text_sm().text_color(theme.accent).child(text.to_string()))
}

fn event_log(sig: Signal<String>, msg: &'static str) -> impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut App) {
    move |_, _, cx: &mut App| { sig.set(cx, msg.into()); }
}

// ── Button samples ────────────────────────────────────────────────────────

fn button_sample(state: &GalleryState) -> impl IntoElement {
    let log = state.overlay_event.clone();
    div().flex().flex_col().gap_3()
        .child(strip()
            .child(Button::new("btn-pri", "Primary").primary().on_click(event_log(log.clone(), "Primary")))
            .child(Button::new("btn-sec", "Secondary").variant(ButtonVariant::Secondary).on_click(event_log(log.clone(), "Secondary")))
            .child(Button::new("btn-ghost", "Ghost").ghost().on_click(event_log(log.clone(), "Ghost")))
            .child(Button::new("btn-dang", "Danger").danger().on_click(event_log(log.clone(), "Danger"))))
        .child(strip()
            .child(Button::new("btn-pri-icon", "With Icon").primary().icon(IconName::Play).on_click(event_log(log.clone(), "Play")))
            .child(Button::new("btn-dis", "Disabled").primary().disabled(true)))
        .child(strip()
            .child(Button::new("btn-sec-icon", "Refresh").variant(ButtonVariant::Secondary).icon(IconName::RefreshCw).on_click(event_log(log.clone(), "Refresh")))
            .child(Button::new("btn-ghost-icon", "Settings").ghost().icon(IconName::Settings).on_click(event_log(log, "Settings"))))
}

fn icon_button_sample(state: &GalleryState) -> impl IntoElement {
    let log = state.overlay_event.clone();
    div().flex().flex_col().gap_2()
        .child(strip()
            .child(IconButton::new("ib-plus", IconName::Plus).size(IconSize::Small).on_click(event_log(log.clone(), "Plus")))
            .child(IconButton::new("ib-search", IconName::Search).size(IconSize::Small).on_click(event_log(log.clone(), "Search icon")))
            .child(IconButton::new("ib-archive", IconName::Archive).size(IconSize::Small).on_click(event_log(log.clone(), "Archive")))
            .child(IconButton::new("ib-panel", IconName::PanelLeft).size(IconSize::Small).active(true).on_click(event_log(log, "Panel toggle"))))
        .child(strip()
            .child(IconButton::new("ib-dis", IconName::Plus).size(IconSize::Small).disabled(true)))
}

// ── Choice samples ────────────────────────────────────────────────────────

fn choice_sample(state: &GalleryState) -> impl IntoElement {
    div().flex().flex_col().gap_3()
        .child(strip()
            .child(Toggle::bound("demo-toggle", state.notifications.clone()).label("Notifications"))
            .child(Checkbox::bound("demo-check", state.auto_archive.clone()).label("Auto archive")))
        .child(strip()
            .child(Radio::bound("demo-radio-sys", state.radio_choice.clone(), "system", "System"))
            .child(Radio::bound("demo-radio-light", state.radio_choice.clone(), "light", "Light"))
            .child(Radio::bound("demo-radio-dark", state.radio_choice.clone(), "dark", "Dark")))
}

// ── Text input samples ────────────────────────────────────────────────────

fn text_input_sample(state: &GalleryState, focused: bool) -> impl IntoElement {
    div().flex().flex_col().gap_3()
        .child(div().max_w(px(320.0)).child(
            TextInput::bound("demo-name", state.name_focus.clone(), state.name_input.clone())
                .placeholder("Enter your name")
                .focused(focused)))
        .child(div().max_w(px(320.0)).child(
            TextInput::new("demo-dis", state.name_focus.clone(), &TextInputState::with_text("Disabled input"))
                .disabled(true)))
        .child(div().text_xs().child("TextInput accepts and displays text via binding"))
}

fn search_sample(state: &GalleryState) -> impl IntoElement {
    let log = state.overlay_event.clone();
    div().flex().flex_col().gap_3()
        .child(div().max_w(px(280.0)).child(
            SearchField::bound("demo-search", state.search_focus.clone(), state.search_input.clone())
                .placeholder("Filter items..."),
        ))
        .child(strip()
            .child(FilterBar::new("demo-fb")
                .child(FilterChip::new("chip-all", "All").icon(IconName::LayoutGrid).on_click(event_log(log.clone(), "Filter: All")))
                .child(FilterChip::new("chip-run", "running").icon(IconName::Play).selected(true))
                .child(FilterChip::new("chip-ro", "readonly").disabled(true))))
        .child(div().text_xs().child("Click chips to filter — all interactive"))
}

// ── Number / Slider / Stepper ─────────────────────────────────────────────

fn number_sample(state: &GalleryState) -> impl IntoElement {
    div().flex().flex_col().gap_3()
        .child(strip()
            .child(NumberInput::bound("demo-num", state.ui_font_size.clone()).range(10, 24).suffix("px"))
            .child(Slider::bound("demo-slider", state.contrast.clone(), 0.0, 100.0)))
        .child(strip()
            .child(Stepper::bound("demo-step", state.ui_font_size.clone()).range(10, 24)))
}

// ── Segmented control ─────────────────────────────────────────────────────

fn segmented_sample(state: &GalleryState) -> impl IntoElement {
    SegmentedControl::bound(
        "demo-seg",
        vec![
            Segment::new("files", "Files"),
            Segment::new("diff", "Diff"),
            Segment::new("review", "Review"),
        ],
        state.seg_tab.clone(),
    )
}

// ── Disclosure ────────────────────────────────────────────────────────────

fn disclosure_sample(state: &GalleryState, open: bool) -> impl IntoElement {
    div().flex().flex_col().gap_2()
        .child(Disclosure::bound("demo-disc", "Advanced settings", state.core_disclosure_open.clone())
            .detail("Expand to configure"))
        .when(open, |this| {
            this.child(div().pl_4().child(
                Checkbox::bound("demo-disc-opt", state.auto_archive.clone()).label("Auto archive")))
        })
}

// ── List rows ─────────────────────────────────────────────────────────────

fn list_sample(theme: Theme) -> impl IntoElement {
    div().flex().flex_col().gap_1()
        .child(NavRow::new("demo-nav", IconName::Terminal, "Active session").count(3).selected(true))
        .child(NavRow::new("demo-nav2", IconName::PanelLeft, "Files"))
        .child(Divider::horizontal())
        .child(TreeRow::new("demo-tree", IconName::Folder, "src/components").expandable(true))
        .child(TreeRow::new("demo-tree-file", IconName::FileText, "button.rs").depth(1).selected(true))
}
