//! Core kit gallery — every component wired to relay signals and actually working.

use gpui::{App, Entity, IntoElement, ParentElement, Styled, div, prelude::FluentBuilder, px};
use relay::Signal;
use relay_uikit::{
    ActiveTheme, Badge, Button, ButtonVariant, Checkbox, CountBadge, Disclosure, Divider,
    FilterBar, FilterChip, Icon, IconButton, IconName, IconSize, Label, LabelSize, ListItem,
    NavRow, NumberInput, Radio, SearchField, SegmentedControl, Segment, Slider, Stepper,
    TextInput, TextInputState, Theme, Toggle, TreeRow,
};

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
    let disclosure_open = state.core_disclosure_open.get(cx);
    let name_focused = state.name_focus.is_focused(window);

    let global_event = state.overlay_event.get(cx);
    let s0 = section(cx, "Event log (global)", event_display(theme, &global_event));
    let s1 = section(cx, "Buttons", button_sample(state));
    let s2 = section(cx, "Icon Buttons", icon_button_sample(state));
    let s3 = section(cx, "Toggle · Checkbox · Radio", choice_sample(state));
    let s4 = section(cx, "Text Input", text_input_sample(state, name_focused));
    let s5 = section(cx, "Search & Filter", search_sample(state));
    let s6 = section(cx, "Number & Slider & Stepper", number_sample(state));
    let s7 = section(cx, "Segmented Control", segmented_sample(state));
    let s8 = section(cx, "Disclosure", disclosure_sample(state, disclosure_open));
    let s9 = section(cx, "List & Tree Rows", tree_sample(state, cx));

    div()
        .flex()
        .flex_col()
        .gap(px(relay_uikit::space::XL))
        .child(s0).child(s1).child(s2).child(s3).child(s4)
        .child(s5).child(s6).child(s7).child(s8).child(s9)
}

// ── Event helpers ─────────────────────────────────────────────────────────

fn event_display(theme: Theme, text: &str) -> impl IntoElement {
    div()
        .px_3().py_2().rounded(px(relay_uikit::radius::MD))
        .bg(theme.panel).border_1().border_color(theme.border)
        .flex().items_center().gap_2()
        .child(Label::new("Last event:").size(LabelSize::XSmall))
        .child(div().text_sm().text_color(theme.accent).child(text.to_string()))
}

fn log_click(sig: Signal<String>, msg: &'static str) -> impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut App) {
    move |_, _, cx: &mut App| { sig.set(cx, msg.into()); }
}

// ── Button samples ────────────────────────────────────────────────────────

fn button_sample(state: &GalleryState) -> impl IntoElement {
    let log = state.overlay_event.clone();
    div().flex().flex_col().gap_3()
        .child(strip()
            .child(Button::new("btn-pri", "Primary").primary().on_click(log_click(log.clone(), "Primary")))
            .child(Button::new("btn-sec", "Secondary").variant(ButtonVariant::Secondary).on_click(log_click(log.clone(), "Secondary")))
            .child(Button::new("btn-ghost", "Ghost").ghost().on_click(log_click(log.clone(), "Ghost")))
            .child(Button::new("btn-dang", "Danger").danger().on_click(log_click(log.clone(), "Danger"))))
        .child(strip()
            .child(Button::new("btn-pri-icon", "With Icon").primary().icon(IconName::Play).on_click(log_click(log.clone(), "Play")))
            .child(Button::new("btn-dis", "Disabled").primary().disabled(true)))
        .child(strip()
            .child(Button::new("btn-sec-icon", "Refresh").variant(ButtonVariant::Secondary).icon(IconName::RefreshCw).on_click(log_click(log.clone(), "Refresh")))
            .child(Button::new("btn-ghost-icon", "Settings").ghost().icon(IconName::Settings).on_click(log_click(log, "Settings"))))
}

fn icon_button_sample(state: &GalleryState) -> impl IntoElement {
    let log = state.overlay_event.clone();
    div().flex().flex_col().gap_2()
        .child(strip()
            .child(IconButton::new("ib-plus", IconName::Plus).size(IconSize::Small).on_click(log_click(log.clone(), "Plus")))
            .child(IconButton::new("ib-search", IconName::Search).size(IconSize::Small).on_click(log_click(log.clone(), "Search")))
            .child(IconButton::new("ib-archive", IconName::Archive).size(IconSize::Small).on_click(log_click(log.clone(), "Archive")))
            .child(IconButton::new("ib-panel", IconName::PanelLeft).size(IconSize::Small).active(true).on_click(log_click(log, "Panel toggle"))))
        .child(strip()
            .child(IconButton::new("ib-dis", IconName::Plus).size(IconSize::Small).disabled(true)))
}

// ── Choice samples ────────────────────────────────────────────────────────

fn choice_sample(state: &GalleryState) -> impl IntoElement {
    let log = state.overlay_event.clone();
    div().flex().flex_col().gap_3()
        .child(strip()
            .child(Toggle::bound("demo-toggle", state.notifications.clone()).label("Notifications").on_click(log_click(log.clone(), "Toggle toggled")))
            .child(Checkbox::bound("demo-check", state.auto_archive.clone()).label("Auto archive").on_click(log_click(log.clone(), "Checkbox toggled"))))
        .child(strip()
            .child(Radio::bound("demo-radio-sys", state.radio_choice.clone(), "system", "System").on_click(log_click(log.clone(), "Radio: system")))
            .child(Radio::bound("demo-radio-light", state.radio_choice.clone(), "light", "Light").on_click(log_click(log.clone(), "Radio: light")))
            .child(Radio::bound("demo-radio-dark", state.radio_choice.clone(), "dark", "Dark").on_click(log_click(log.clone(), "Radio: dark"))))
}

// ── Text input samples ────────────────────────────────────────────────────

fn text_input_sample(state: &GalleryState, focused: bool) -> impl IntoElement {
    div().flex().flex_col().gap_3()
        .child(div().max_w(px(320.0)).child(
            TextInput::bound("demo-name", state.name_focus.clone(), state.name_input.clone())
                .placeholder("Enter your name — type to see live text")
                .focused(focused)))
        .child(div().max_w(px(320.0)).child(
            TextInput::new("demo-dis", state.name_focus.clone(), &TextInputState::with_text("Disabled input"))
                .disabled(true)))
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
                .child(FilterChip::new("chip-all", "All").icon(IconName::LayoutGrid).on_click(log_click(log.clone(), "Filter: All")))
                .child(FilterChip::new("chip-run", "running").icon(IconName::Play).selected(true).on_click(log_click(log.clone(), "Filter: running")))
                .child(FilterChip::new("chip-ro", "readonly").disabled(true))))
}

// ── Number / Slider / Stepper ─────────────────────────────────────────────

fn number_sample(state: &GalleryState) -> impl IntoElement {
    let log = state.overlay_event.clone();
    div().flex().flex_col().gap_3()
        .child(strip()
            .child(NumberInput::bound("demo-num", state.ui_font_size.clone())
                .range(10, 24)
                .suffix("px")
                .on_change({
                    let log = log.clone();
                    move |v: i32, _: &mut gpui::Window, cx: &mut App| {
                        log.set(cx, format!("Font size: {v}px").into());
                    }
                })))
        .child(strip()
            .child(Slider::bound("demo-slider", state.contrast.clone(), 0.0, 100.0)
                .on_change({
                    let log = log.clone();
                    move |v: f32, _: &mut gpui::Window, cx: &mut App| {
                        log.set(cx, format!("Contrast: {:.0}%", v).into());
                    }
                })))
        .child(strip()
            .child(Stepper::bound("demo-step", state.ui_font_size.clone())
                .range(10, 24)
                .on_decrement(log_click(log.clone(), "Stepper: decremented"))
                .on_increment(log_click(log.clone(), "Stepper: incremented"))))
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
            .detail("Click or press Space to expand"))
        .when(open, |this| {
            this.child(div().pl_4().flex().flex_col().gap_2()
                .child(Label::new("These settings appear when the disclosure is open.").size(LabelSize::Small))
                .child(Checkbox::bound("demo-disc-opt", state.auto_archive.clone()).label("Auto archive")))
        })
}

// ── List & Tree rows ──────────────────────────────────────────────────────

fn tree_sample(state: &GalleryState, cx: &App) -> impl IntoElement {
    div().flex().flex_col().gap_1()
        .child(NavRow::new("demo-nav", IconName::Terminal, "Active session").count(3).selected(true))
        .child(NavRow::new("demo-nav2", IconName::PanelLeft, "Files"))
        .child(Divider::horizontal())
        .child(TreeRow::new("demo-tree", IconName::Folder, "src/components").expandable(true))
        .child(
            TreeRow::bound("demo-tree-file", IconName::FileText, "button.rs", state.core_disclosure_open.clone())
                .depth(1)
                .expanded_bound(state.core_disclosure_open.clone()),
        )
        .when(state.core_disclosure_open.get(cx), |this| {
            this.child(TreeRow::new("demo-tree-nested", IconName::FileText, "mod.rs").depth(2))
        })
}
