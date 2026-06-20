use gpui::{
    Anchor, AnyElement, App, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled,
    Window, anchored, deferred, div, px,
};

use relay_ui_core::{interaction::DismissHandler, theme};

/// Anchored floating content with window-edge snapping.
#[derive(IntoElement)]
pub struct Overlay {
    content: AnyElement,
    top: f32,
    left: f32,
    anchor: Anchor,
    on_dismiss: Option<DismissHandler>,
}

/// Build an [`Overlay`] around floating content.
pub fn overlay(content: impl IntoElement) -> Overlay {
    Overlay {
        content: content.into_any_element(),
        top: 0.0,
        left: 0.0,
        anchor: Anchor::TopLeft,
        on_dismiss: None,
    }
}

impl Overlay {
    /// Offset from the anchor corner, in pixels.
    pub fn offset(mut self, left: f32, top: f32) -> Self {
        self.left = left;
        self.top = top;
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Overlay {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let on_dismiss = self.on_dismiss;
        deferred(
            anchored()
                .snap_to_window_with_margin(px(theme::OVERLAY_WINDOW_MARGIN))
                .anchor(self.anchor)
                .child({
                    let mut container = div()
                        .absolute()
                        .left(px(self.left))
                        .top(px(self.top))
                        .occlude()
                        .child(self.content);
                    if let Some(on_dismiss) = on_dismiss {
                        let dismiss_for_key = std::rc::Rc::new(on_dismiss);
                        let dismiss_for_mouse = dismiss_for_key.clone();
                        container = container
                            .on_mouse_down_out(move |_event, window, cx| {
                                dismiss_for_mouse(window, cx);
                            })
                            .key_context("Overlay")
                            .on_key_down(move |event: &gpui::KeyDownEvent, window, cx| {
                                if event.keystroke.key.as_str() == "escape" {
                                    dismiss_for_key(window, cx);
                                    cx.stop_propagation();
                                }
                            });
                    }
                    container
                }),
        )
        .with_priority(theme::OVERLAY_PRIORITY_FLOATING)
    }
}
