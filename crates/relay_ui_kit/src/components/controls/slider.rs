use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
    relative,
};

use crate::theme::{ActiveTheme, radius};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// A compact horizontal slider with optional step callbacks.
#[derive(IntoElement)]
pub struct Slider {
    id: ElementId,
    value: f32,
    min: f32,
    max: f32,
    on_decrement: Option<ClickHandler>,
    on_increment: Option<ClickHandler>,
}

impl Slider {
    pub fn new(id: impl Into<ElementId>, value: f32, min: f32, max: f32) -> Self {
        Self {
            id: id.into(),
            value,
            min,
            max,
            on_decrement: None,
            on_increment: None,
        }
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

    pub fn ratio(&self) -> f32 {
        slider_ratio(self.value, self.min, self.max)
    }
}

impl RenderOnce for Slider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let ratio = self.ratio();
        let on_decrement = self.on_decrement;
        let on_increment = self.on_increment;

        div()
            .id(self.id)
            .h(px(28.0))
            .flex()
            .items_center()
            .gap_2()
            .child(step_button(
                "slider-decrement",
                "-",
                on_decrement,
                theme.hover,
            ))
            .child(
                div()
                    .relative()
                    .w(px(168.0))
                    .h(px(16.0))
                    .flex()
                    .items_center()
                    .child(
                        div()
                            .h(px(3.0))
                            .w_full()
                            .rounded(px(radius::SM))
                            .bg(theme.border)
                            .child(
                                div()
                                    .h_full()
                                    .w(relative(ratio))
                                    .rounded(px(radius::SM))
                                    .bg(theme.accent),
                            ),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(relative(ratio))
                            .size(px(14.0))
                            .ml(px(-7.0))
                            .rounded(px(7.0))
                            .bg(theme.text)
                            .border_1()
                            .border_color(theme.panel),
                    ),
            )
            .child(step_button(
                "slider-increment",
                "+",
                on_increment,
                theme.hover,
            ))
    }
}

fn step_button(
    id: &'static str,
    label: &'static str,
    handler: Option<ClickHandler>,
    hover_bg: gpui::Hsla,
) -> impl IntoElement {
    div()
        .id(id)
        .size(px(22.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(radius::MD))
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
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

fn slider_ratio(value: f32, min: f32, max: f32) -> f32 {
    if max <= min {
        return 0.0;
    }

    ((value - min) / (max - min)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_ratio_clamps_overflow() {
        assert_eq!(slider_ratio(150.0, 0.0, 100.0), 1.0);
    }
}
