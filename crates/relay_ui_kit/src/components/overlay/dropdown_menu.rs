use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled,
    Window, div, prelude::FluentBuilder,
};

use super::{Menu, MenuItem, overlay};

#[derive(IntoElement)]
pub struct DropdownMenu {
    id: ElementId,
    trigger: AnyElement,
    items: Vec<MenuItem>,
    open: bool,
    min_width: f32,
    left: f32,
    top: f32,
}

impl DropdownMenu {
    pub fn new(id: impl Into<ElementId>, trigger: impl IntoElement, items: Vec<MenuItem>) -> Self {
        Self {
            id: id.into(),
            trigger: trigger.into_any_element(),
            items,
            open: false,
            min_width: 180.0,
            left: 0.0,
            top: 34.0,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
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
}

impl RenderOnce for DropdownMenu {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let menu_id = (self.id.clone(), "menu");

        div()
            .id(self.id)
            .relative()
            .child(self.trigger)
            .when(self.open, |this| {
                this.child(
                    overlay(Menu::new(menu_id, self.items).min_width(self.min_width))
                        .offset(self.left, self.top),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropdown_menu_defaults_to_closed() {
        let menu = DropdownMenu::new("dropdown", div(), vec![]);

        assert!(!menu.open);
    }
}
