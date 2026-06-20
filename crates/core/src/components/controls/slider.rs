use gpui::{
    App, AppContext as _, ClickEvent, DragMoveEvent, ElementId, Empty, InteractiveElement,
    IntoElement, MouseButton, ParentElement, RenderOnce, Role, StatefulInteractiveElement, Styled,
    Window, div, prelude::FluentBuilder, px, relative,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{ClickHandler, SharedChangeHandler},
    theme::{ActiveTheme, radius},
};

/// A compact horizontal slider with optional step callbacks.
#[derive(IntoElement)]
pub struct Slider {
    id: ElementId,
    value: f32,
    min: f32,
    max: f32,
    on_decrement: Option<ClickHandler>,
    on_increment: Option<ClickHandler>,
    on_change: Option<SharedChangeHandler<f32>>,
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
            on_change: None,
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

    pub fn on_change(mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::rc::Rc::new(handler));
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
        let on_change = self.on_change;
        let drag = DraggedSlider {
            id: self.id.clone(),
        };
        let min = self.min;
        let max = self.max;

        div()
            .id(self.id.clone())
            .h(px(28.0))
            .flex()
            .items_center()
            .gap_2()
            .role(Role::Slider)
            .child(step_button(
                "slider-decrement",
                IconName::Minus,
                on_decrement,
                theme.hover,
                theme.text_muted,
            ))
            .child(
                div()
                    .id((self.id.clone(), "track"))
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
                            .id((self.id.clone(), "thumb"))
                            .absolute()
                            .left(relative(ratio))
                            .size(px(16.0))
                            .ml(px(-8.0))
                            .rounded_full()
                            .bg(theme.panel)
                            .border_2()
                            .border_color(theme.accent),
                    )
                    .when_some(on_change.clone(), |this, handler| {
                        let drag_for_start = drag.clone();
                        this.cursor_pointer()
                            .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                                window.prevent_default();
                            })
                            .on_drag(drag_for_start, move |_, _, _window, cx| cx.new(|_| Empty))
                            .on_drag_move::<DraggedSlider>(move |event, window, cx| {
                                if event.drag(cx).id != drag.id {
                                    return;
                                }
                                handler(value_from_drag(event, min, max), window, cx);
                                cx.stop_propagation();
                            })
                    }),
            )
            .child(step_button(
                "slider-increment",
                IconName::Plus,
                on_increment,
                theme.hover,
                theme.text_muted,
            ))
    }
}

#[derive(Clone)]
struct DraggedSlider {
    id: ElementId,
}

fn step_button(
    id: &'static str,
    icon: IconName,
    handler: Option<ClickHandler>,
    hover_bg: gpui::Hsla,
    color: gpui::Hsla,
) -> impl IntoElement {
    div()
        .id(id)
        .size(px(22.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(radius::MD))
        .when_some(handler, |this, handler| {
            this.cursor_pointer()
                .hover(move |style| style.bg(hover_bg))
                .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                    window.prevent_default();
                })
                .on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
        })
        .child(Icon::new(icon).size(IconSize::Small).color(color))
}

fn slider_ratio(value: f32, min: f32, max: f32) -> f32 {
    if max <= min {
        return 0.0;
    }

    ((value - min) / (max - min)).clamp(0.0, 1.0)
}

fn value_from_drag(event: &DragMoveEvent<DraggedSlider>, min: f32, max: f32) -> f32 {
    if max <= min {
        return min;
    }

    let width = f32::from(event.bounds.size.width).max(1.0);
    let x = f32::from(event.event.position.x - event.bounds.left());
    min + (x / width).clamp(0.0, 1.0) * (max - min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_ratio_clamps_overflow() {
        assert_eq!(slider_ratio(150.0, 0.0, 100.0), 1.0);
    }

    #[test]
    fn slider_starts_without_change_handler() {
        let slider = Slider::new("slider", 50.0, 0.0, 100.0);

        assert!(slider.on_change.is_none());
    }
}
