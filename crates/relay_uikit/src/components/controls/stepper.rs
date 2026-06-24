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

#[derive(Clone, Copy)]
struct StepperRange {
    min: Option<i32>,
    max: Option<i32>,
}

impl StepperRange {
    fn clamp(self, value: i32, delta: i32) -> i32 {
        let new_value = if delta >= 0 {
            value.saturating_add(delta)
        } else {
            value.saturating_sub(-delta)
        };
        let mut clamped = new_value;
        if let Some(min) = self.min {
            clamped = clamped.max(min);
        }
        if let Some(max) = self.max {
            clamped = clamped.min(max);
        }
        clamped
    }

    fn at_limit(self, value: i32, delta: i32) -> bool {
        if delta < 0 {
            self.min.is_some_and(|min| value <= min)
        } else {
            self.max.is_some_and(|max| value >= max)
        }
    }
}

#[derive(Clone, Copy)]
struct StepperButtonStyle {
    hover_bg: gpui::Hsla,
    color: gpui::Hsla,
}

struct StepperButtonProps {
    id: ElementId,
    icon: IconName,
    binding: Option<Binding<i32>>,
    delta: i32,
    handler: Option<ClickHandler>,
    range: StepperRange,
    disabled: bool,
    style: StepperButtonStyle,
}

struct StepperValueProps {
    value: String,
    binding: Option<Binding<i32>>,
    range: StepperRange,
    disabled: bool,
    text_color: gpui::Hsla,
    border_color: gpui::Hsla,
    hover_bg: gpui::Hsla,
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
        let Self {
            id,
            value,
            disabled,
            min,
            max,
            binding,
            on_decrement,
            on_increment,
            on_reset,
        } = self;
        let theme = *cx.theme();
        let range = StepperRange { min, max };
        let display = binding
            .as_ref()
            .map_or(value, |stepper_binding| stepper_binding.get(cx).to_string());
        let button_style = StepperButtonStyle {
            hover_bg: theme.hover,
            color: theme.text_muted,
        };

        div()
            .id(id.clone())
            .h(px(30.0))
            .flex()
            .items_center()
            .gap_1()
            .when(disabled, |this| {
                this.opacity(DISABLED_OPACITY)
                    .cursor(gpui::CursorStyle::OperationNotAllowed)
            })
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
                    .child(stepper_button(StepperButtonProps {
                        id: (id.clone(), "decrement").into(),
                        icon: IconName::Minus,
                        binding: binding.clone(),
                        delta: -1,
                        handler: on_decrement,
                        range,
                        disabled,
                        style: button_style,
                    }))
                    .child(stepper_value(StepperValueProps {
                        value: display,
                        binding: binding.clone(),
                        range,
                        disabled,
                        text_color: theme.text,
                        border_color: theme.border,
                        hover_bg: theme.hover,
                    }))
                    .child(stepper_button(StepperButtonProps {
                        id: (id.clone(), "increment").into(),
                        icon: IconName::Plus,
                        binding,
                        delta: 1,
                        handler: on_increment,
                        range,
                        disabled,
                        style: button_style,
                    })),
            )
            .when_some(on_reset, |this, handler| {
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

fn clamp_value(value: i32, delta: i32, range: StepperRange) -> i32 {
    range.clamp(value, delta)
}

fn stepper_button(props: StepperButtonProps) -> impl IntoElement {
    let StepperButtonProps {
        id,
        icon,
        binding,
        delta,
        handler,
        range,
        disabled,
        style,
    } = props;
    let at_limit = binding.as_ref().is_some_and(|stepper_binding| {
        let value = stepper_binding.signal().peek(|current| *current);
        range.at_limit(value, delta)
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
            this.opacity(DISABLED_OPACITY)
                .cursor(gpui::CursorStyle::OperationNotAllowed)
        })
        .when(interactive, |this| {
            this.cursor_pointer()
                .hover(move |hover| hover.bg(style.hover_bg))
                .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                    window.prevent_default();
                })
                .on_click(move |event, window, cx| {
                    if let Some(stepper_binding) = &binding {
                        stepper_binding.update(cx, |value| {
                            *value = clamp_value(*value, delta, range);
                            true
                        });
                    }
                    if let Some(handler) = &handler {
                        handler(event, window, cx);
                    }
                    cx.stop_propagation();
                })
        })
        .child(Icon::new(icon).size(IconSize::Small).color(style.color))
}

fn stepper_value(props: StepperValueProps) -> impl IntoElement {
    let StepperValueProps {
        value,
        binding,
        range,
        disabled,
        text_color,
        border_color,
        hover_bg,
    } = props;
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
                    let Some(delta) = stepper_key_delta(key) else {
                        return;
                    };
                    if let Some(binding) = &binding {
                        binding.update(cx, |value| {
                            *value = clamp_value(*value, delta, range);
                            true
                        });
                        cx.stop_propagation();
                    }
                })
        })
        .child(value)
}

fn stepper_key_delta(key: &str) -> Option<i32> {
    match key {
        "up" => Some(1),
        "down" => Some(-1),
        _ => None,
    }
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
        let range = StepperRange {
            min: Some(0),
            max: Some(10),
        };
        let unbounded = StepperRange {
            min: None,
            max: None,
        };

        assert_eq!(clamp_value(5, -10, range), 0);
        assert_eq!(clamp_value(8, 5, range), 10);
        assert_eq!(clamp_value(5, 2, unbounded), 7);
        assert_eq!(clamp_value(5, -3, unbounded), 2);
    }

    #[test]
    fn stepper_key_delta_uses_gpui_arrow_key_names() {
        assert_eq!(stepper_key_delta("up"), Some(1));
        assert_eq!(stepper_key_delta("down"), Some(-1));
        assert_eq!(stepper_key_delta("arrow-up"), None);
        assert_eq!(stepper_key_delta("arrow-down"), None);
    }
}
