use gpui::{
    Anchor, AnyElement, App, InteractiveElement, IntoElement, ParentElement, RenderOnce, Window,
    anchored, deferred, div, point, px,
};

use crate::{interaction::DismissHandler, theme};

/// Anchored floating content with window-edge snapping.
#[derive(IntoElement)]
pub struct Overlay {
    content: AnyElement,
    top: f32,
    left: f32,
    anchor: Anchor,
    window_corner_inset: Option<f32>,
    on_dismiss: Option<DismissHandler>,
}

/// Build an [`Overlay`] around floating content.
pub fn overlay(content: impl IntoElement) -> Overlay {
    Overlay {
        content: content.into_any_element(),
        top: 0.0,
        left: 0.0,
        anchor: Anchor::TopLeft,
        window_corner_inset: None,
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

    /// Position the floating content against a window corner with a uniform inset.
    pub fn window_corner(mut self, anchor: Anchor, inset: f32) -> Self {
        self.anchor = anchor;
        self.window_corner_inset = Some(inset);
        self
    }

    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Overlay {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let on_dismiss = self.on_dismiss;
        let mut anchored = anchored()
            .snap_to_window_with_margin(px(theme::OVERLAY_WINDOW_MARGIN))
            .anchor(self.anchor);

        if let Some(inset) = self.window_corner_inset {
            let viewport = window.viewport_size();
            let position = match self.anchor {
                Anchor::TopLeft => point(px(inset), px(inset)),
                Anchor::TopRight => point(viewport.width - px(inset), px(inset)),
                Anchor::BottomLeft => point(px(inset), viewport.height - px(inset)),
                Anchor::BottomRight => {
                    point(viewport.width - px(inset), viewport.height - px(inset))
                }
                Anchor::TopCenter => point(viewport.width / 2.0, px(inset)),
                Anchor::BottomCenter => point(viewport.width / 2.0, viewport.height - px(inset)),
                Anchor::LeftCenter => point(px(inset), viewport.height / 2.0),
                Anchor::RightCenter => point(viewport.width - px(inset), viewport.height / 2.0),
            };
            anchored = anchored.position(position);
        } else {
            anchored = anchored.offset(point(px(self.left), px(self.top)));
        }

        deferred(anchored.child({
            let mut container = div().occlude().child(self.content);
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
        }))
        .with_priority(theme::OVERLAY_PRIORITY_FLOATING)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_can_anchor_to_window_corner() {
        let overlay = overlay(div()).window_corner(Anchor::BottomRight, 16.0);

        assert_eq!(overlay.anchor, Anchor::BottomRight);
        assert_eq!(overlay.window_corner_inset, Some(16.0));
    }
}
