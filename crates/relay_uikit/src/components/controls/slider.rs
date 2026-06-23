use gpui::{
    App, AppContext as _, ClickEvent, DragMoveEvent, ElementId, Empty, InteractiveElement,
    IntoElement, KeyDownEvent, MouseButton, ParentElement, RenderOnce, Role,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px, relative,
};
use relay::Binding;

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{ClickHandler, SharedChangeHandler},
    theme::{ActiveTheme, DISABLED_OPACITY, radius},
};

/// A compact horizontal slider with optional step callbacks.
#[derive(IntoElement)]
pub struct Slider {
    id: ElementId,
    value: f32,
    min: f32,
    max: f32,
    disabled: bool,
    on_decrement: Option<ClickHandler>,
    on_increment: Option<ClickHandler>,
    binding: Option<Binding<f32>>,
    on_change: Option<SharedChangeHandler<f32>>,
}

impl Slider {
    pub fn new(id: impl Into<ElementId>, value: f32, min: f32, max: f32) -> Self {
        Self {
            id: id.into(),
            value,
            min,
            max,
            disabled: false,
            on_decrement: None,
            on_increment: None,
            binding: None,
            on_change: None,
        }
    }

    pub fn bound(id: impl Into<ElementId>, binding: Binding<f32>, min: f32, max: f32) -> Self {
        Self {
            id: id.into(),
            value: min,
            min,
            max,
            disabled: false,
            on_decrement: None,
            on_increment: None,
            binding: Some(binding),
            on_change: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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
        let binding = self.binding;
        let value = binding
            .as_ref()
            .map_or(self.value, |binding| binding.get(cx));
        let ratio = slider_ratio(value, self.min, self.max);
        let on_decrement = self.on_decrement;
        let on_increment = self.on_increment;
        let on_change = self.on_change;
        let can_change = binding.is_some() || on_change.is_some();
        let drag = DraggedSlider {
            id: self.id.clone(),
        };
        let min = self.min;
        let max = self.max;
        let step = (max - min) / 10.0;

        // Auto-wire step buttons for bound() sliders
        let dec_handler = if binding.is_some() {
            let binding = binding.clone();
            let handler = on_change.clone();
            let on_decrement = on_decrement;
            Some(Box::new(
                move |event: &ClickEvent, window: &mut Window, cx: &mut App| {
                    if let Some(binding) = &binding {
                        binding.update(cx, |v| {
                            *v = (*v - step).max(min);
                            true
                        });
                    }
                    if let Some(handler) = &handler {
                        handler(
                            binding.as_ref().map_or(value - step, |b| b.get(cx)),
                            window,
                            cx,
                        );
                    }
                    if let Some(h) = &on_decrement {
                        h(event, window, cx);
                    }
                },
            ) as ClickHandler)
        } else {
            on_decrement
        };

        let inc_handler = if binding.is_some() {
            let binding = binding.clone();
            let handler = on_change.clone();
            let on_increment = on_increment;
            Some(Box::new(
                move |event: &ClickEvent, window: &mut Window, cx: &mut App| {
                    if let Some(binding) = &binding {
                        binding.update(cx, |v| {
                            *v = (*v + step).min(max);
                            true
                        });
                    }
                    if let Some(handler) = &handler {
                        handler(
                            binding.as_ref().map_or(value + step, |b| b.get(cx)),
                            window,
                            cx,
                        );
                    }
                    if let Some(h) = &on_increment {
                        h(event, window, cx);
                    }
                },
            ) as ClickHandler)
        } else {
            on_increment
        };

        div()
            .id(self.id.clone())
            .h(px(28.0))
            .flex()
            .items_center()
            .gap_2()
            .role(Role::Slider)
            .when(self.disabled, |this| {
                this.opacity(DISABLED_OPACITY)
                    .cursor(gpui::CursorStyle::OperationNotAllowed)
            })
            .child(step_button(
                "slider-decrement",
                IconName::Minus,
                dec_handler,
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
                    .tab_index(0)
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
                    .when(can_change, |this| {
                        let drag_for_start = drag.clone();
                        let binding = binding.clone();
                        let handler = on_change.clone();
                        this.cursor_pointer()
                            .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                                window.prevent_default();
                            })
                            .on_drag(drag_for_start, move |_, _, _window, cx| cx.new(|_| Empty))
                            .on_drag_move::<DraggedSlider>(move |event, window, cx| {
                                if event.drag(cx).id != drag.id {
                                    return;
                                }
                                let value = value_from_drag(event, min, max);
                                if let Some(binding) = &binding {
                                    binding.set(cx, value);
                                }
                                if let Some(handler) = &handler {
                                    handler(value, window, cx);
                                }
                                cx.stop_propagation();
                            })
                    })
                    .when(can_change && !self.disabled, |this| {
                        let binding = binding.clone();
                        let handler = on_change.clone();
                        this.on_key_down(move |event: &KeyDownEvent, window, cx| {
                            let key = event.keystroke.key.as_str();
                            if key == "arrow-left" || key == "arrow-right" {
                                let current = binding.as_ref().map_or(value, |b| b.get(cx));
                                let new_value = if key == "arrow-left" {
                                    (current - step).max(min)
                                } else {
                                    (current + step).min(max)
                                };
                                if let Some(binding) = &binding {
                                    binding.set(cx, new_value);
                                }
                                if let Some(handler) = &handler {
                                    handler(new_value, window, cx);
                                }
                                cx.stop_propagation();
                            }
                        })
                    }),
            )
            .child(step_button(
                "slider-increment",
                IconName::Plus,
                inc_handler,
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
    if max <= min || !min.is_finite() || !max.is_finite() {
        return min;
    }

    let width = f32::from(event.bounds.size.width).max(1.0);
    let x = f32::from(event.event.position.x - event.bounds.left());
    let ratio = (x / width).clamp(0.0, 1.0);
    let value = min + ratio * (max - min);
    if value.is_finite() { value } else { min }
}

#[cfg(test)]
mod tests {
    use relay::ReactiveAppExt;

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

    #[test]
    fn bound_slider_stores_binding() {
        let mut app = gpui::TestApp::new();
        let slider = app.update(|cx| Slider::bound("slider", cx.binding(50.0), 0.0, 100.0));

        assert!(slider.binding.is_some());
    }

    #[test]
    fn slider_disabled_defaults_to_false() {
        let slider = Slider::new("slider", 50.0, 0.0, 100.0);

        assert!(!slider.disabled);
    }
}
