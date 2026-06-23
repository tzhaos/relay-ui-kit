use std::hash::Hash;

use gpui::{
    App, ClickEvent, ElementId, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, RenderOnce, Role, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::{Binding, Selector, WindowSignalExt};

use crate::{
    interaction::{ActionHandler, SelectionSource},
    theme::{ActiveTheme, DISABLED_OPACITY, radius, space},
};

/// One labelled segment in a [`SegmentedControl`].
pub struct Segment<K> {
    pub key: K,
    pub label: String,
}

impl<K> Segment<K> {
    pub fn new(key: K, label: impl Into<String>) -> Self {
        Self {
            key,
            label: label.into(),
        }
    }
}

/// A pill-grouped segmented control.
#[derive(IntoElement)]
pub struct SegmentedControl<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    id: ElementId,
    segments: Vec<Segment<K>>,
    active: Option<K>,
    disabled: bool,
    selection: Option<SelectionSource<K>>,
    on_select: Option<ActionHandler<K>>,
}

impl<K> SegmentedControl<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    pub fn new(id: impl Into<ElementId>, segments: Vec<Segment<K>>) -> Self {
        Self {
            id: id.into(),
            segments,
            active: None,
            disabled: false,
            selection: None,
            on_select: None,
        }
    }

    pub fn bound(id: impl Into<ElementId>, segments: Vec<Segment<K>>, binding: Binding<K>) -> Self {
        Self {
            id: id.into(),
            segments,
            active: None,
            disabled: false,
            selection: Some(SelectionSource::binding(binding)),
            on_select: None,
        }
    }

    pub fn active(mut self, active: K) -> Self {
        self.active = Some(active);
        self
    }

    pub fn selected_with(mut self, selection: SelectionSource<K>) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn selected_by(self, selector: Selector<K>) -> Self {
        self.selected_with(SelectionSource::selector(selector))
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select(mut self, handler: impl Fn(K, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl<K> RenderOnce for SegmentedControl<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let Self {
            id,
            segments,
            active,
            disabled,
            selection,
            on_select,
        } = self;
        let selection = selection.or_else(|| {
            active.clone().map(|active| {
                SelectionSource::binding(window.use_binding(
                    (id.clone(), "active-segment"),
                    cx,
                    || active,
                ))
            })
        });
        let handler = on_select.map(std::rc::Rc::new);
        let active = selection
            .as_ref()
            .and_then(|selection| selection.get(cx))
            .or(active);
        let interactive = !disabled && (selection.is_some() || handler.is_some());
        let segment_keys: Vec<K> = segments.iter().map(|segment| segment.key.clone()).collect();

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
            .when(interactive, |this| {
                this.tab_index(0)
                    .focus_visible(move |style| style.border_color(theme.accent_border))
            })
            .when(disabled, |this| {
                this.opacity(DISABLED_OPACITY)
                    .cursor(gpui::CursorStyle::OperationNotAllowed)
            });

        for (index, segment) in segments.into_iter().enumerate() {
            let is_active = active.as_ref().is_some_and(|active| segment.key == *active);
            let key = segment.key.clone();
            let handler = handler.clone();
            let selection = selection.clone();
            let cell = div()
                .id(("segment", index))
                .h_full()
                .px_3()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(radius::SM))
                .role(Role::RadioButton)
                .aria_selected(is_active)
                .text_xs()
                .font_weight(if is_active {
                    gpui::FontWeight::SEMIBOLD
                } else {
                    gpui::FontWeight::MEDIUM
                })
                .when(is_active, |this| {
                    this.bg(theme.panel).text_color(theme.text)
                })
                .when(!is_active, |this| this.text_color(theme.text_muted))
                .when(interactive && !is_active, |this| {
                    this.cursor_pointer()
                        .hover(move |style| style.text_color(theme.text_secondary))
                        .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                            window.prevent_default();
                        })
                        .on_click(move |_: &ClickEvent, window, cx| {
                            if let Some(selection) = &selection {
                                selection.select(cx, key.clone());
                            }
                            if let Some(handler) = &handler {
                                handler(key.clone(), window, cx);
                            }
                            cx.stop_propagation();
                        })
                })
                .child(segment.label);
            row = row.child(cell);
        }

        if interactive {
            let selection = selection.clone();
            let handler = handler.clone();
            row = row.on_key_down(move |event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "arrow-left" || key == "arrow-right" {
                    let current = selection
                        .as_ref()
                        .and_then(|selection| selection.get(cx))
                        .or_else(|| active.clone());
                    let current_idx = current
                        .as_ref()
                        .and_then(|current| {
                            segment_keys
                                .iter()
                                .position(|segment_key| segment_key == current)
                        })
                        .unwrap_or(0);
                    let next_idx = if key == "arrow-left" {
                        if current_idx == 0 {
                            segment_keys.len().saturating_sub(1)
                        } else {
                            current_idx - 1
                        }
                    } else if current_idx + 1 >= segment_keys.len() {
                        0
                    } else {
                        current_idx + 1
                    };
                    let next_key = segment_keys[next_idx].clone();
                    if let Some(selection) = &selection {
                        selection.select(cx, next_key.clone());
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

        assert!(control.selection.is_some());
    }

    #[test]
    fn segmented_control_disabled_defaults_to_false() {
        let control = SegmentedControl::new("segmented", vec![Segment::new("one", "One")]);

        assert!(!control.disabled);
    }
}
