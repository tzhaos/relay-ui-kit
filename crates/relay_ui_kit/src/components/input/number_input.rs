use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    interaction::ClickHandler,
    theme::{ActiveTheme, radius},
};

/// A compact numeric input with optional stepper callbacks.
#[derive(IntoElement)]
pub struct NumberInput {
    id: ElementId,
    value: i32,
    suffix: Option<String>,
    on_decrement: Option<ClickHandler>,
    on_increment: Option<ClickHandler>,
}

impl NumberInput {
    pub fn new(id: impl Into<ElementId>, value: i32) -> Self {
        Self {
            id: id.into(),
            value,
            suffix: None,
            on_decrement: None,
            on_increment: None,
        }
    }

    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
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
}

impl RenderOnce for NumberInput {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .id(self.id)
            .h(px(30.0))
            .min_w(px(108.0))
            .pl_2()
            .flex()
            .items_center()
            .rounded(px(radius::MD))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex_1()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text)
                    .child(self.value.to_string()),
            )
            .when_some(self.suffix, |this, suffix| {
                this.child(
                    div()
                        .px_1()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child(suffix),
                )
            })
            .child(
                div()
                    .h_full()
                    .ml_1()
                    .border_l_1()
                    .border_color(theme.border)
                    .flex()
                    .items_center()
                    .child(stepper(
                        "number-decrement",
                        "-",
                        self.on_decrement,
                        theme.hover,
                        theme.text_muted,
                    ))
                    .child(div().h(px(16.0)).w(px(1.0)).bg(theme.border.opacity(0.7)))
                    .child(stepper(
                        "number-increment",
                        "+",
                        self.on_increment,
                        theme.hover,
                        theme.text_muted,
                    )),
            )
    }
}

fn stepper(
    id: &'static str,
    label: &'static str,
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
        .text_size(px(11.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(color)
        .when_some(handler, |this, handler| {
            this.cursor_pointer()
                .hover(move |style| style.bg(hover_bg))
                .on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
        })
        .child(label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_input_keeps_optional_suffix() {
        let input = NumberInput::new("number", 14).suffix("px");

        assert_eq!(input.suffix.as_deref(), Some("px"));
    }
}
