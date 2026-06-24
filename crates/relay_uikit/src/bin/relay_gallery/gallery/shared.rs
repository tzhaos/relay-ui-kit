use gpui::{App, FocusHandle, FontWeight, IntoElement, ParentElement, Styled, div, px};
use relay::Binding;
use relay_uikit::{ActiveTheme, IconName, TextInput, TextInputState, radius, space};

pub(super) fn section<T: IntoElement>(cx: &App, title: &str, body: T) -> impl IntoElement + use<T> {
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

pub(super) struct TextInputFieldProps<'a> {
    pub id: &'static str,
    pub input: &'a Binding<TextInputState>,
    pub focus: FocusHandle,
    pub focused: bool,
    pub icon: Option<IconName>,
    pub placeholder: &'static str,
}

pub(super) fn text_input_field(props: TextInputFieldProps<'_>) -> impl IntoElement {
    let TextInputFieldProps {
        id,
        input,
        focus,
        focused,
        icon,
        placeholder,
    } = props;
    let mut field = TextInput::bound(id, focus, input.clone())
        .placeholder(placeholder)
        .focused(focused);
    if let Some(icon) = icon {
        field = field.leading_icon(icon);
    }
    field
}
