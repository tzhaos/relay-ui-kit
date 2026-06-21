use gpui::{
    App, ClickEvent, ElementId, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, RenderOnce, Role, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    interaction::SelectHandler,
    theme::{ActiveTheme, DISABLED_OPACITY, radius, space},
};

/// One labelled segment in a [`SegmentedControl`].
pub struct Segment {
    pub key: &'static str,
    pub label: &'static str,
}

impl Segment {
    pub fn new(key: &'static str, label: &'static str) -> Self {
        Self { key, label }
    }
}

/// A pill-grouped segmented control.
#[derive(IntoElement)]
pub struct SegmentedControl {
    id: ElementId,
    segments: Vec<Segment>,
    active: &'static str,
    disabled: bool,
    binding: Option<Binding<&'static str>>,
    on_select: Option<SelectHandler>,
}

impl SegmentedControl {
    pub fn new(id: impl Into<ElementId>, segments: Vec<Segment>) -> Self {
        Self {
            id: id.into(),
            segments,
            active: "",
            disabled: false,
            binding: None,
            on_select: None,
        }
    }

    pub fn bound(
        id: impl Into<ElementId>,
        segments: Vec<Segment>,
        binding: Binding<&'static str>,
    ) -> Self {
        Self {
            id: id.into(),
            segments,
            active: "",
            disabled: false,
            binding: Some(binding),
            on_select: None,
        }
    }

    pub fn active(mut self, active: &'static str) -> Self {
        self.active = active;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&'static str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SegmentedControl {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let binding = self.binding;
        let handler = self.on_select.map(std::rc::Rc::new);
        let active = binding
            .as_ref()
            .map_or(self.active, |binding| binding.get(cx));
        let disabled = self.disabled;
        let interactive = !disabled && (binding.is_some() || handler.is_some());
        let segment_keys: Vec<&'static str> = self.segments.iter().map(|s| s.key).collect();
        let segments = self.segments;
        let id = self.id;

        let mut row = div()
            .id(id.clone())
            .h(px(28.0))
            .p(px(space::XXS))
            .flex()
            .items_center()
            .gap(px(space::XXS))
            .rounded(px(radius::MD))
            .bg(theme.inset)
            .border_1()
            .border_color(theme.border)
            .role(Role::RadioGroup)
            .tab_index(0)
            .when(disabled, |this| this.opacity(DISABLED_OPACITY));

        for (index, segment) in segments.into_iter().enumerate() {
            let is_active = segment.key == active;
            let key = segment.key;
            let handler = handler.clone();
            let binding = binding.clone();
            let cell = div()
                .id(("segment", index))
                .h_full()
                .px_3()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(radius::SM))
                .text_xs()
                .font_weight(if is_active {
                    gpui::FontWeight::SEMIBOLD
                } else {
                    gpui::FontWeight::MEDIUM
                })
                .when(is_active, |this| {
                    this.bg(theme.panel).text_color(theme.text)
                })
                .when(!is_active, |this| {
                    this.text_color(theme.text_muted)
                        .cursor_pointer()
                        .hover(move |style| style.text_color(theme.text_secondary))
                })
                .when(
                    interactive && !is_active,
                    |this| {
                        this.on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                            window.prevent_default();
                        })
                        .on_click(move |_: &ClickEvent, window, cx| {
                            if let Some(binding) = &binding {
                                binding.set(cx, key);
                            }
                            if let Some(handler) = &handler {
                                handler(key, window, cx);
                            }
                            cx.stop_propagation();
                        })
                    },
                )
                .child(segment.label);
            row = row.child(cell);
        }

        if interactive {
            let binding = binding.clone();
            let handler = handler.clone();
            row = row.on_key_down(move |event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "arrow-left" || key == "arrow-right" {
                    let current = binding
                        .as_ref()
                        .map_or(active, |b| b.get(cx));
                    let current_idx = segment_keys
                        .iter()
                        .position(|&k| k == current)
                        .unwrap_or(0);
                    let next_idx = if key == "arrow-left" {
                        if current_idx == 0 {
                            segment_keys.len().saturating_sub(1)
                        } else {
                            current_idx - 1
                        }
                    } else {
                        if current_idx + 1 >= segment_keys.len() {
                            0
                        } else {
                            current_idx + 1
                        }
                    };
                    let next_key = segment_keys[next_idx];
                    if let Some(binding) = &binding {
                        binding.set(cx, next_key);
                    }
                    if let Some(handler) = &handler {
                        handler(next_key, window, cx);
                    }
                    cx.stop_propagation();
                }
            });
        }

        row
    }
}

#[cfg(test)]
mod tests {
    use gpui::TestApp;
    use relay::ReactiveAppExt;

    use super::*;

    #[test]
    fn bound_segmented_control_stores_binding() {
        let mut app = TestApp::new();
        let control = app.update(|cx| {
            SegmentedControl::bound(
                "segmented",
                vec![Segment::new("one", "One")],
                cx.binding("one"),
            )
        });

        assert!(control.binding.is_some());
    }

    #[test]
    fn segmented_control_disabled_defaults_to_false() {
        let control = SegmentedControl::new("segmented", vec![Segment::new("one", "One")]);

        assert!(!control.disabled);
    }
}
