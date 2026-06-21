use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::ClickHandler,
    theme::{ActiveTheme, radius},
};

/// A compact segmented numeric control for zoom, font size, or density settings.
#[derive(IntoElement)]
pub struct Stepper {
    id: ElementId,
    value: String,
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
            binding: Some(binding),
            on_decrement: None,
            on_increment: None,
            on_reset: None,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
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
                        theme.hover,
                        theme.text_muted,
                    ))
                    .child(stepper_value(display, theme.text, theme.border))
                    .child(stepper_button(
                        (id.clone(), "increment"),
                        IconName::Plus,
                        binding,
                        1,
                        self.on_increment,
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

fn stepper_button(
    id: impl Into<ElementId>,
    icon: IconName,
    binding: Option<Binding<i32>>,
    delta: i32,
    handler: Option<ClickHandler>,
    hover_bg: gpui::Hsla,
    color: gpui::Hsla,
) -> impl IntoElement {
    let interactive = binding.is_some() || handler.is_some();
    div()
        .id(id)
        .w(px(30.0))
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .when(interactive, |this| {
            this.cursor_pointer()
                .hover(move |style| style.bg(hover_bg))
                .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                    window.prevent_default();
                })
                .on_click(move |event, window, cx| {
                    if let Some(binding) = &binding {
                        binding.update(cx, |value| {
                            *value += delta;
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
    text_color: gpui::Hsla,
    border_color: gpui::Hsla,
) -> impl IntoElement {
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
}
