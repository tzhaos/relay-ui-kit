use gpui::{Anchor, App, ElementId, IntoElement, RenderOnce, Window};

use crate::interaction::DismissHandler;

use super::{Menu, MenuItem, overlay};

/// A right-click or more-actions menu placed relative to its anchor.
#[derive(IntoElement)]
pub struct ContextMenu {
    id: ElementId,
    items: Vec<MenuItem>,
    min_width: f32,
    left: f32,
    top: f32,
    anchor: Anchor,
    on_dismiss: Option<DismissHandler>,
}

impl ContextMenu {
    pub fn new(id: impl Into<ElementId>, items: Vec<MenuItem>) -> Self {
        Self {
            id: id.into(),
            items,
            min_width: 180.0,
            left: 0.0,
            top: 0.0,
            anchor: Anchor::TopLeft,
            on_dismiss: None,
        }
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

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

impl RenderOnce for ContextMenu {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut ov = overlay(Menu::new(self.id, self.items).min_width(self.min_width))
            .anchor(self.anchor)
            .offset(self.left, self.top);

        if let Some(handler) = self.on_dismiss {
            ov = ov.on_dismiss(handler);
        } else {
            // Always wire dismiss so overlay closes on click-outside / Escape
            ov = ov.on_dismiss(|_window, _cx| {});
        }

        ov
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_menu_default_offset_is_zero() {
        let menu = ContextMenu::new("context", vec![]);

        assert_eq!(menu.left, 0.0);
    }
}
