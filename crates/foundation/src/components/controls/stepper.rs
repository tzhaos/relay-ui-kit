use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

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
    on_decrement: Option<ClickHandler>,
    on_increment: Option<ClickHandler>,
    on_reset: Option<ClickHandler>,
}

impl Stepper {
    pub fn new(id: impl Into<ElementId>, value: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            on_decrement: None,
            on_increment: None,
            on_reset: None,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    crate::callback_builder!(on_decrement, on_decrement, ClickEvent);

    crate::callback_builder!(on_increment, on_increment, ClickEvent);

    crate::callback_builder!(on_reset, on_reset, ClickEvent);
}

impl RenderOnce for Stepper {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let id = self.id;
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
                        self.on_decrement,
                        theme.hover,
                        theme.text_muted,
                    ))
                    .child(stepper_value(self.value, theme.text, theme.border))
                    .child(stepper_button(
                        (id.clone(), "increment"),
                        IconName::Plus,
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
    handler: Option<ClickHandler>,
    hover_bg: gpui::Hsla,
    color: gpui::Hsla,
) -> impl IntoElement {
    div()
        .id(id)
        .w(px(30.0))
        .h_full()
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
