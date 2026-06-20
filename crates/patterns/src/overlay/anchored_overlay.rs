use gpui::{
    Anchor, AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId,
    InteractiveElement, IntoElement, LayoutId, Length, ParentElement, Pixels, Point, Position,
    RenderOnce, Style, Window, anchored, deferred, div, point, px, relative, size,
};

use relay_ui_core::{interaction::DismissHandler, theme};

/// Trigger-anchored floating content.
#[derive(IntoElement)]
pub struct AnchoredOverlay {
    id: ElementId,
    trigger: AnyElement,
    content: Option<AnyElement>,
    open: bool,
    anchor: Anchor,
    attach: Anchor,
    offset: Point<Pixels>,
    full_width: bool,
    on_dismiss: Option<DismissHandler>,
}

#[derive(Default)]
struct AnchoredOverlayState {
    trigger_bounds: Option<Bounds<Pixels>>,
}

pub struct AnchoredOverlayLayoutState {
    trigger: AnyElement,
    trigger_layout_id: LayoutId,
    content: Option<AnyElement>,
}

impl AnchoredOverlay {
    pub fn new(
        id: impl Into<ElementId>,
        trigger: impl IntoElement,
        content: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            trigger: trigger.into_any_element(),
            content: Some(content.into_any_element()),
            open: false,
            anchor: Anchor::TopLeft,
            attach: Anchor::BottomLeft,
            offset: point(px(0.0), px(0.0)),
            full_width: false,
            on_dismiss: None,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Defines which corner of the floating content is placed at the attachment point.
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Defines which corner of the trigger supplies the attachment point.
    pub fn attach(mut self, attach: Anchor) -> Self {
        self.attach = attach;
        self
    }

    /// Pixel adjustment applied after trigger attachment is resolved.
    pub fn offset(mut self, offset: Point<Pixels>) -> Self {
        self.offset = offset;
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for AnchoredOverlay {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        self
    }
}

impl Element for AnchoredOverlay {
    type RequestLayoutState = AnchoredOverlayLayoutState;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let Some(global_id) = id else {
            let trigger_layout_id = self.trigger.request_layout(window, cx);
            let layout_id = window.request_layout(Style::default(), Some(trigger_layout_id), cx);
            return (
                layout_id,
                AnchoredOverlayLayoutState {
                    trigger: core::mem::replace(&mut self.trigger, div().into_any()),
                    trigger_layout_id,
                    content: None,
                },
            );
        };

        window.with_element_state(
            global_id,
            |element_state: Option<AnchoredOverlayState>, window| {
                let element_state = element_state.unwrap_or_default();
                let trigger_layout_id = self.trigger.request_layout(window, cx);
                let mut content = None;
                let mut content_layout_id = None;

                if self.open
                    && let (Some(trigger_bounds), Some(content_element)) =
                        (element_state.trigger_bounds, self.content.take())
                {
                    let mut container = div().occlude().child(content_element);
                    if let Some(on_dismiss) = self.on_dismiss.take() {
                        let dismiss_for_key = std::rc::Rc::new(on_dismiss);
                        let dismiss_for_mouse = dismiss_for_key.clone();
                        container = container
                            .on_mouse_down_out(move |_event, window, cx| {
                                dismiss_for_mouse(window, cx);
                            })
                            .key_context("AnchoredOverlay")
                            .on_key_down(move |event: &gpui::KeyDownEvent, window, cx| {
                                if event.keystroke.key.as_str() == "escape" {
                                    dismiss_for_key(window, cx);
                                    cx.stop_propagation();
                                }
                            });
                    }

                    let anchored_content = deferred(
                        anchored()
                            .snap_to_window_with_margin(px(theme::OVERLAY_WINDOW_MARGIN))
                            .anchor(self.anchor)
                            .position(trigger_bounds.corner(self.attach) + self.offset)
                            .child(container),
                    )
                    .with_priority(theme::OVERLAY_PRIORITY_FLOATING)
                    .into_any();

                    content = Some(anchored_content);
                    content_layout_id = content
                        .as_mut()
                        .map(|element| element.request_layout(window, cx));
                }

                let mut style = Style {
                    position: Position::Relative,
                    ..Style::default()
                };
                if self.full_width {
                    style.size = size(relative(1.0).into(), Length::Auto);
                    style.min_size = size(Length::Definite(px(0.0).into()), Length::Auto);
                }

                let layout_id = window.request_layout(
                    style,
                    content_layout_id.into_iter().chain(Some(trigger_layout_id)),
                    cx,
                );

                (
                    (
                        layout_id,
                        AnchoredOverlayLayoutState {
                            trigger: core::mem::replace(&mut self.trigger, div().into_any()),
                            trigger_layout_id,
                            content,
                        },
                    ),
                    element_state,
                )
            },
        )
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        request_layout.trigger.prepaint(window, cx);

        if let Some(content) = request_layout.content.as_mut() {
            content.prepaint(window, cx);
        }

        let trigger_bounds = window.layout_bounds(request_layout.trigger_layout_id);
        if let Some(global_id) = id {
            window.with_element_state(global_id, |element_state, _window| {
                let mut element_state: AnchoredOverlayState = element_state.unwrap_or_default();
                element_state.trigger_bounds = Some(trigger_bounds);
                ((), element_state)
            });
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        request_layout.trigger.paint(window, cx);
        if let Some(content) = request_layout.content.as_mut() {
            content.paint(window, cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchored_overlay_defaults_to_trigger_bottom_left() {
        let overlay = AnchoredOverlay::new("overlay", div(), div());

        assert_eq!(overlay.anchor, Anchor::TopLeft);
        assert_eq!(overlay.attach, Anchor::BottomLeft);
    }
}
