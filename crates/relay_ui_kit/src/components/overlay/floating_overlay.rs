use gpui::{
    AnyElement, App, Corner, IntoElement, ParentElement, RenderOnce, Styled, Window, anchored,
    deferred, div, px,
};

/// Anchored floating content with window-edge snapping.
#[derive(IntoElement)]
pub struct Overlay {
    content: AnyElement,
    top: f32,
    left: f32,
    corner: Corner,
    on_dismiss: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
}

/// Build an [`Overlay`] around floating content.
pub fn overlay(content: impl IntoElement) -> Overlay {
    Overlay {
        content: content.into_any_element(),
        top: 0.0,
        left: 0.0,
        corner: Corner::TopLeft,
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

    pub fn anchor(mut self, corner: Corner) -> Self {
        self.corner = corner;
        self
    }

    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Overlay {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let _on_dismiss = self.on_dismiss;
        deferred(
            anchored()
                .snap_to_window_with_margin(px(8.0))
                .anchor(self.corner)
                .child(
                    div()
                        .absolute()
                        .left(px(self.left))
                        .top(px(self.top))
                        .child(self.content),
                ),
        )
        .with_priority(1)
    }
}
