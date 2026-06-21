use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::ClickHandler,
    theme::{ActiveTheme, DISABLED_OPACITY, radius},
};

/// A compact segmented numeric control for zoom, font size, or density settings.
#[derive(IntoElement)]
pub struct Stepper {
    id: ElementId,
    value: String,
    disabled: bool,
    min: Option<i32>,
    max: Option<i32>,
    binding: Option<Binding<i32>>,
    on_decrement: Option<ClickHandler>,
    on_increment: Option<ClickHandler>,
    on_reset: Option<ClickHandler>,
}

impl Stepper {
    pub fn new(id: impl Into<ElementId>, value: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            disabled: false,
            min: None,
            max: None,
            binding: None,
            on_decrement: None,
            on_increment: None,
            on_reset: None,
        }
    }

    pub fn bound(id: impl Into<ElementId>, binding: Binding<i32>) -> Self {
        let value = binding.signal().peek(|v| v.to_string());
        Self {
            id: id.into(),
            value,
            disabled: false,
            min: None,
            max: None,
            binding: Some(binding),
            on_decrement: None,
            on_increment: None,
            on_reset: None,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn range(mut self, min: i32, max: i32) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self
    }

    pub fn on_decrement(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_decrement = Some(Box::new(handler));
        self
    }

    pub fn on_increment(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_increment = Some(Box::new(handler));
        self
    }

    pub fn on_reset(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_reset = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Stepper {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let id = self.id;
        let binding = self.binding;
        let disabled = self.disabled;
        let min = self.min;
        let max = self.max;
        let display = binding
            .as_ref()
            .map_or(self.value.clone(), |b| format!("{}", b.get(cx)));
        let reset = self.on_reset;

        div()
            .id(id.clone())
            .h(px(30.0))
            .flex()
            .items_center()
            .gap_1()
            .when(disabled, |this| this.opacity(DISABLED_OPACITY).cursor(gpui::CursorStyle::OperationNotAllowed))
            .child(
                div()
                    .h_full()
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .rounded(px(radius::MD))
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.panel_alt)
                    .child(stepper_button(
                        (id.clone(), "decrement"),
                        IconName::Minus,
                        binding.clone(),
                        -1,
                        self.on_decrement,
                        min,
                        max,
                        disabled,
                        theme.hover,
                        theme.text_muted,
                    ))
                    .child(stepper_value(
                        display,
                        binding.clone(),
                        min,
                        max,
                        disabled,
                        theme.text,
                        theme.border,
                        theme.hover,
                    ))
                    .child(stepper_button(
                        (id.clone(), "increment"),
                        IconName::Plus,
                        binding,
                        1,
                        self.on_increment,
                        min,
                        max,
                        disabled,
                        theme.hover,
                        theme.text_muted,
                    )),
            )
            .when_some(reset, |this, handler| {
                this.child(
                    div()
                        .id((id, "reset"))
                        .size(px(30.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(radius::MD))
                        .text_color(theme.text_muted)
                        .cursor_pointer()
                        .hover(move |style| style.bg(theme.hover).text_color(theme.text))
                        .child(Icon::new(IconName::RefreshCw).size(IconSize::XSmall))
                        .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                            window.prevent_default();
                        })
                        .on_click(move |event, window, cx| {
                            handler(event, window, cx);
                            cx.stop_propagation();
                        }),
                )
            })
    }
}

fn clamp_value(value: i32, delta: i32, min: Option<i32>, max: Option<i32>) -> i32 {
    let new_value = if delta >= 0 {
        value.saturating_add(delta as i32)
    } else {
        value.saturating_sub((-delta) as i32)
    };
    let mut clamped = new_value;
    if let Some(min) = min {
        clamped = clamped.max(min);
    }
    if let Some(max) = max {
        clamped = clamped.min(max);
    }
    clamped
}

fn stepper_button(
    id: impl Into<ElementId>,
    icon: IconName,
    binding: Option<Binding<i32>>,
    delta: i32,
    handler: Option<ClickHandler>,
    min: Option<i32>,
    max: Option<i32>,
    disabled: bool,
    hover_bg: gpui::Hsla,
    color: gpui::Hsla,
) -> impl IntoElement {
    let at_limit = binding.as_ref().map_or(false, |b| {
        let value = b.signal().peek(|v| *v);
        if delta < 0 {
            min.is_some_and(|m| value <= m)
        } else {
            max.is_some_and(|m| value >= m)
        }
    });
    let interactive = !disabled && !at_limit && (binding.is_some() || handler.is_some());
    let is_dimmed = disabled || at_limit;
    div()
        .id(id)
        .w(px(30.0))
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .when(is_dimmed, |this| {
            this.opacity(DISABLED_OPACITY).cursor(gpui::CursorStyle::OperationNotAllowed)
        })
        .when(interactive, |this| {
            this.cursor_pointer()
                .hover(move |style| style.bg(hover_bg))
                .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                    window.prevent_default();
                })
                .on_click(move |event, window, cx| {
                    if let Some(binding) = &binding {
                        binding.update(cx, |value| {
                            *value = clamp_value(*value, delta, min, max);
                            true
                        });
                    }
                    if let Some(handler) = &handler {
                        handler(event, window, cx);
                    }
                    cx.stop_propagation();
                })
        })
        .child(Icon::new(icon).size(IconSize::Small).color(color))
}

fn stepper_value(
    value: String,
    binding: Option<Binding<i32>>,
    min: Option<i32>,
    max: Option<i32>,
    disabled: bool,
    text_color: gpui::Hsla,
    border_color: gpui::Hsla,
    hover_bg: gpui::Hsla,
) -> impl IntoElement {
    let interactive = !disabled && binding.is_some();
    div()
        .min_w(px(58.0))
        .h_full()
        .px_2()
        .flex()
        .items_center()
        .justify_center()
        .border_l_1()
        .border_r_1()
        .border_color(border_color)
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .text_color(text_color)
        .when(interactive, |this| {
            let binding = binding.clone();
            this.tab_index(0)
                .cursor_pointer()
                .hover(move |style| style.bg(hover_bg))
                .on_key_down(move |event: &KeyDownEvent, _window, cx| {
                    let key = event.keystroke.key.as_str();
                    let delta: i32 = match key {
                        "arrow-up" => 1,
                        "arrow-down" => -1,
                        _ => return,
                    };
                    if let Some(binding) = &binding {
                        binding.update(cx, |value| {
                            *value = clamp_value(*value, delta, min, max);
                            true
                        });
                        cx.stop_propagation();
                    }
                })
        })
        .child(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stepper_keeps_display_value() {
        let stepper = Stepper::new("zoom-stepper", "100%");

        assert_eq!(stepper.value(), "100%");
    }

    #[test]
    fn stepper_disabled_defaults_to_false() {
        let stepper = Stepper::new("zoom-stepper", "100%");

        assert!(!stepper.disabled);
    }

    #[test]
    fn clamp_value_respects_min_max() {
        assert_eq!(clamp_value(5, -10, Some(0), Some(10)), 0);
        assert_eq!(clamp_value(8, 5, Some(0), Some(10)), 10);
        assert_eq!(clamp_value(5, 2, None, None), 7);
        assert_eq!(clamp_value(5, -3, None, None), 2);
    }
}
