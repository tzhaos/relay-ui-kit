use gpui::{
    App, ClickEvent, ElementId, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    KeyDownEvent, MouseButton, ParentElement, RenderOnce, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{ClickHandler, KeyHandler},
    theme::{ActiveTheme, BORDER_WIDTH, radius},
};

use super::TextInputState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberInputLayout {
    ControlsAroundValue,
    ControlsTrailing,
}

struct EditableNumberValue {
    focus: FocusHandle,
    before: String,
    after: String,
    focused: bool,
    key_context: &'static str,
    on_key: Option<KeyHandler>,
}

/// A compact numeric input with optional stepper callbacks.
#[derive(IntoElement)]
pub struct NumberInput {
    id: ElementId,
    value: i32,
    suffix: Option<String>,
    layout: NumberInputLayout,
    editable: Option<EditableNumberValue>,
    on_decrement: Option<ClickHandler>,
    on_increment: Option<ClickHandler>,
}

impl NumberInput {
    pub fn new(id: impl Into<ElementId>, value: i32) -> Self {
        Self {
            id: id.into(),
            value,
            suffix: None,
            layout: NumberInputLayout::ControlsAroundValue,
            editable: None,
            on_decrement: None,
            on_increment: None,
        }
    }

    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    pub fn layout(mut self, layout: NumberInputLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn editable(mut self, focus: FocusHandle, state: &TextInputState) -> Self {
        let (before, after) = state.split();
        self.editable = Some(EditableNumberValue {
            focus,
            before: before.to_string(),
            after: after.to_string(),
            focused: false,
            key_context: "NumberInput",
            on_key: None,
        });
        self
    }

    pub fn input(self, focus: FocusHandle, state: &TextInputState) -> Self {
        self.editable(focus, state)
    }

    pub fn focused(mut self, focused: bool) -> Self {
        if let Some(editable) = &mut self.editable {
            editable.focused = focused;
        }
        self
    }

    pub fn key_context(mut self, key_context: &'static str) -> Self {
        if let Some(editable) = &mut self.editable {
            editable.key_context = key_context;
        }
        self
    }

    pub fn on_key(
        mut self,
        handler: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        if let Some(editable) = &mut self.editable {
            editable.on_key = Some(Box::new(handler));
        }
        self
    }

    crate::callback_builder!(on_decrement, on_decrement, ClickEvent);

    crate::callback_builder!(on_increment, on_increment, ClickEvent);
}

impl RenderOnce for NumberInput {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let NumberInput {
            id,
            value: current_value,
            suffix,
            layout,
            editable,
            on_decrement,
            on_increment,
        } = self;
        let theme = *cx.theme();
        let value_id = format!("{id}-value");
        let value = match editable {
            Some(editable) => {
                editable_number_value(value_id, editable, current_value, suffix, theme)
                    .into_any_element()
            }
            None => {
                number_value(current_value, suffix, theme.text, theme.text_muted).into_any_element()
            }
        };
        let decrement = stepper(
            (id.clone(), "decrement"),
            IconName::Minus,
            on_decrement,
            theme.hover,
            theme.text_muted,
        );
        let increment = stepper(
            (id.clone(), "increment"),
            IconName::Plus,
            on_increment,
            theme.hover,
            theme.text_muted,
        );

        div()
            .id(id)
            .h(px(30.0))
            .min_w(px(108.0))
            .flex()
            .items_center()
            .overflow_hidden()
            .rounded(px(radius::MD))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border)
            .map(|this| match layout {
                NumberInputLayout::ControlsAroundValue => {
                    this.child(decrement).child(value).child(increment)
                }
                NumberInputLayout::ControlsTrailing => this.child(value).child(
                    div()
                        .h_full()
                        .border_l_1()
                        .border_color(theme.border)
                        .flex()
                        .items_center()
                        .child(decrement)
                        .child(
                            div()
                                .h(px(16.0))
                                .w(px(BORDER_WIDTH))
                                .bg(theme.border.opacity(0.7)),
                        )
                        .child(increment),
                ),
            })
    }
}

fn number_value(
    value: i32,
    suffix: Option<String>,
    text_color: gpui::Hsla,
    suffix_color: gpui::Hsla,
) -> impl IntoElement {
    div()
        .min_w(px(58.0))
        .h_full()
        .px_2()
        .flex()
        .items_center()
        .justify_center()
        .gap_1()
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .text_color(text_color)
        .child(value.to_string())
        .when_some(suffix, |this, suffix| {
            this.child(div().text_xs().text_color(suffix_color).child(suffix))
        })
}

fn editable_number_value(
    id: impl Into<ElementId>,
    editable: EditableNumberValue,
    fallback: i32,
    suffix: Option<String>,
    theme: crate::Theme,
) -> impl IntoElement {
    let focus_for_click = editable.focus.clone();
    let focus_for_mouse_down = editable.focus.clone();
    let on_key = editable.on_key;
    let show_fallback =
        editable.before.is_empty() && editable.after.is_empty() && !editable.focused;

    div()
        .id(id)
        .min_w(px(58.0))
        .h_full()
        .px_2()
        .flex()
        .items_center()
        .justify_center()
        .gap_1()
        .border_l_1()
        .border_r_1()
        .border_color(theme.border)
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .track_focus(&editable.focus)
        .tab_index(0)
        .key_context(editable.key_context)
        .cursor(gpui::CursorStyle::IBeam)
        .when(editable.focused, |this| this.bg(theme.accent_bg))
        .when(show_fallback, |this| {
            this.text_color(theme.text_muted)
                .child(fallback.to_string())
        })
        .when(!show_fallback, |this| {
            this.text_color(theme.text)
                .child(editable.before)
                .when(editable.focused, |this| {
                    this.child(div().w(px(1.5)).h(px(16.0)).bg(theme.accent))
                })
                .child(editable.after)
        })
        .when_some(suffix, |this, suffix| {
            this.child(div().text_xs().text_color(theme.text_muted).child(suffix))
        })
        .when_some(on_key, |this, on_key| {
            this.on_key_down(move |event, window, cx| {
                on_key(event, window, cx);
                cx.stop_propagation();
            })
        })
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            window.focus(&focus_for_mouse_down, cx);
        })
        .on_click(move |_, window, cx| {
            window.focus(&focus_for_click, cx);
        })
}

fn stepper(
    id: impl Into<ElementId>,
    icon: IconName,
    handler: Option<ClickHandler>,
    hover_bg: gpui::Hsla,
    color: gpui::Hsla,
) -> impl IntoElement {
    div()
        .id(id)
        .w(px(24.0))
        .h(px(28.0))
        .flex()
        .items_center()
        .justify_center()
        .when_some(handler, |this, handler| {
            this.cursor_pointer()
                .hover(move |style| style.bg(hover_bg))
                .on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
        })
        .child(Icon::new(icon).size(IconSize::Small).color(color))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_input_keeps_optional_suffix() {
        let input = NumberInput::new("number", 14).suffix("px");

        assert_eq!(input.suffix.as_deref(), Some("px"));
    }

    #[test]
    fn number_input_defaults_to_controls_around_value() {
        let input = NumberInput::new("number", 14);

        assert_eq!(input.layout, NumberInputLayout::ControlsAroundValue);
    }

    #[test]
    fn number_input_starts_read_only() {
        let input = NumberInput::new("number", 14);

        assert!(input.editable.is_none());
    }
}
