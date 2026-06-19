use gpui::{Anchor, App, ElementId, IntoElement, RenderOnce, Window};

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
}

impl RenderOnce for ContextMenu {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        overlay(Menu::new(self.id, self.items).min_width(self.min_width))
            .anchor(self.anchor)
            .offset(self.left, self.top)
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
