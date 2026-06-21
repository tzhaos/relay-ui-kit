use gpui::{App, Entity, FocusHandle, FontWeight, IntoElement, ParentElement, Styled, div, px};
use relay::Binding;
use relay_uikit::{
    ActiveTheme, Icon, IconName, IconSize, StatusDot, TextInput, TextInputState, Theme, Tone, radius,
    space,
};

use super::GalleryScenesApp;

pub(super) fn section<T: IntoElement>(
    cx: &App,
    title: &str,
    body: T,
) -> impl IntoElement + use<T> {
    let theme = *cx.theme();
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_size(px(11.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text_muted)
                .child(title.to_uppercase()),
        )
        .child(
            div()
                .p_3()
                .rounded(px(radius::LG))
                .bg(theme.panel)
                .border_1()
                .border_color(theme.border)
                .child(body),
        )
}

pub(super) fn scene_stack() -> gpui::Div {
    div().flex().flex_col().gap(px(space::XL))
}

pub(super) fn strip() -> gpui::Div {
    div().flex().items_center().gap_3().flex_wrap()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn text_input_field(
    _host: &Entity<GalleryScenesApp>,
    id: &'static str,
    input: &Binding<TextInputState>,
    focus: FocusHandle,
    focused: bool,
    icon: Option<IconName>,
    placeholder: &'static str,
) -> impl IntoElement {
    let mut field = TextInput::bound(id, focus, input.clone())
        .placeholder(placeholder)
        .focused(focused);
    if let Some(icon) = icon {
        field = field.leading_icon(icon);
    }
    field
}

pub(super) fn dot_label(theme: Theme, tone: Tone, label: &str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(StatusDot::new(tone))
        .child(
            div()
                .text_sm()
                .text_color(theme.text_secondary)
                .child(label.to_string()),
        )
}

pub(super) fn icon_sample(theme: Theme, name: IconName) -> impl IntoElement {
    div()
        .size(px(32.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(radius::MD))
        .bg(theme.panel_alt)
        .border_1()
        .border_color(theme.border)
        .child(
            Icon::new(name)
                .size(IconSize::Medium)
                .color(theme.text_secondary),
        )
}
