use gpui::{
    Anchor, AnyElement, App, Bounds, Element, ElementId, FocusHandle, GlobalElementId,
    InspectorElementId, InteractiveElement, IntoElement, LayoutId, Length, ParentElement, Pixels,
    Point, Position, Style, Window, anchored, deferred, div, point, px, relative, size,
};

use crate::{interaction::DismissHandler, theme};

/// Trigger-anchored floating content.
pub struct AnchoredOverlay {
    id: ElementId,
    trigger: AnyElement,
    content: Option<AnyElement>,
    open: bool,
    anchor: Anchor,
    attach: Anchor,
    offset: Point<Pixels>,
    full_width: bool,
    focus_handle: Option<FocusHandle>,
    on_dismiss: Option<DismissHandler>,
}

#[derive(Default)]
struct AnchoredOverlayState {
    trigger_bounds: Option<Bounds<Pixels>>,
    previous_focus: Option<FocusHandle>,
    was_open: bool,
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
            focus_handle: None,
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

    pub fn focus_handle(mut self, focus_handle: FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle);
        self
    }

    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }
}

fn sync_overlay_focus_state(
    state: &mut AnchoredOverlayState,
    open: bool,
    focus_handle: Option<&FocusHandle>,
    window: &mut Window,
    cx: &mut App,
) {
    if open && !state.was_open {
        state.previous_focus = window.focused(cx);
        if let Some(focus_handle) = focus_handle.cloned() {
            window.on_next_frame(move |window, _cx| {
                window.on_next_frame(move |window, cx| {
                    window.activate_window();
                    window.focus(&focus_handle, cx);
                });
            });
        }
    } else if !open && state.was_open {
        if let (Some(previous_focus), Some(focus_handle)) =
            (state.previous_focus.as_ref(), focus_handle)
            && focus_handle.contains_focused(window, cx)
        {
            window.focus(previous_focus, cx);
        }
        state.previous_focus = None;
    }

    state.was_open = open;
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
        let overlay_focus = self.focus_handle.clone();

        window.with_element_state(
            global_id,
            |element_state: Option<AnchoredOverlayState>, window| {
                let mut element_state = element_state.unwrap_or_default();
                sync_overlay_focus_state(
                    &mut element_state,
                    self.open,
                    overlay_focus.as_ref(),
                    window,
                    cx,
                );
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

impl IntoElement for AnchoredOverlay {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Context, InteractiveElement, IntoElement, Render, TestApp, Window, div};

    use super::*;

    struct FocusHost {
        trigger_focus: FocusHandle,
        overlay_focus: FocusHandle,
        outside_focus: FocusHandle,
    }

    impl FocusHost {
        fn new(cx: &mut Context<Self>) -> Self {
            Self {
                trigger_focus: cx.focus_handle(),
                overlay_focus: cx.focus_handle(),
                outside_focus: cx.focus_handle(),
            }
        }
    }

    impl Render for FocusHost {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .child(div().track_focus(&self.trigger_focus))
                .child(div().track_focus(&self.overlay_focus))
                .child(div().track_focus(&self.outside_focus))
        }
    }

    #[test]
    fn anchored_overlay_defaults_to_trigger_bottom_left() {
        let overlay = AnchoredOverlay::new("overlay", div(), div());

        assert_eq!(overlay.anchor, Anchor::TopLeft);
        assert_eq!(overlay.attach, Anchor::BottomLeft);
    }

    #[test]
    fn overlay_focus_state_restores_previous_focus_when_overlay_closes() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| FocusHost::new(cx));
        let mut state = AnchoredOverlayState::default();

        window.draw();
        window.update(|view, window, cx| {
            window.activate_window();
            window.focus(&view.trigger_focus, cx);
        });
        window.draw();

        window.update(|view, window, cx| {
            sync_overlay_focus_state(&mut state, true, Some(&view.overlay_focus), window, cx);
        });
        window.draw();
        window.update(|view, window, cx| {
            window.focus(&view.overlay_focus, cx);
        });
        window.draw();

        let overlay_focused =
            window.update(|view, window, cx| view.overlay_focus.contains_focused(window, cx));
        assert!(overlay_focused);

        window.update(|view, window, cx| {
            sync_overlay_focus_state(&mut state, false, Some(&view.overlay_focus), window, cx);
        });
        window.draw();

        let (trigger_focused, overlay_focused) = window.update(|view, window, cx| {
            (
                view.trigger_focus.contains_focused(window, cx),
                view.overlay_focus.contains_focused(window, cx),
            )
        });
        assert!(trigger_focused);
        assert!(!overlay_focused);
    }

    #[test]
    fn overlay_focus_state_does_not_steal_focus_back_after_focus_leaves_overlay() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| FocusHost::new(cx));
        let mut state = AnchoredOverlayState::default();

        window.draw();
        window.update(|view, window, cx| {
            window.activate_window();
            window.focus(&view.trigger_focus, cx);
        });
        window.draw();

        window.update(|view, window, cx| {
            sync_overlay_focus_state(&mut state, true, Some(&view.overlay_focus), window, cx);
        });
        window.draw();
        window.update(|view, window, cx| {
            window.focus(&view.overlay_focus, cx);
        });
        window.draw();

        window.update(|view, window, cx| {
            window.focus(&view.outside_focus, cx);
        });
        window.draw();

        window.update(|view, window, cx| {
            sync_overlay_focus_state(&mut state, false, Some(&view.overlay_focus), window, cx);
        });
        window.draw();

        let (outside_focused, trigger_focused, overlay_focused) =
            window.update(|view, window, cx| {
                (
                    view.outside_focus.contains_focused(window, cx),
                    view.trigger_focus.contains_focused(window, cx),
                    view.overlay_focus.contains_focused(window, cx),
                )
            });
        assert!(outside_focused);
        assert!(!trigger_focused);
        assert!(!overlay_focused);
    }
}
