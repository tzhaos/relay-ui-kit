use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled,
    Window, div, prelude::FluentBuilder,
};

use relay_ui_core::interaction::DismissHandler;

use super::{Menu, MenuItem, overlay};

#[derive(IntoElement)]
pub struct DropdownMenu {
    id: ElementId,
    trigger: AnyElement,
    items: Vec<MenuItem>,
    open: bool,
    auto_dismiss: bool,
    min_width: f32,
    left: f32,
    top: f32,
    on_dismiss: Option<DismissHandler>,
}

impl DropdownMenu {
    pub fn new(id: impl Into<ElementId>, trigger: impl IntoElement, items: Vec<MenuItem>) -> Self {
        Self {
            id: id.into(),
            trigger: trigger.into_any_element(),
            items,
            open: false,
            auto_dismiss: true,
            min_width: 180.0,
            left: 0.0,
            top: 32.0,
            on_dismiss: None,
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

    pub fn auto_dismiss(mut self, auto_dismiss: bool) -> Self {
        self.auto_dismiss = auto_dismiss;
        self
    }

    pub fn offset(mut self, left: f32, top: f32) -> Self {
        self.left = left;
        self.top = top;
        self
    }

    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for DropdownMenu {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let menu_id = (self.id.clone(), "menu");
        let menu = overlay(Menu::new(menu_id, self.items).min_width(self.min_width))
            .offset(self.left, self.top);
        let menu = if self.auto_dismiss {
            if let Some(on_dismiss) = self.on_dismiss {
                menu.on_dismiss(on_dismiss)
            } else {
                menu
            }
        } else {
            menu
        };

        div()
            .id(self.id)
            .relative()
            .child(self.trigger)
            .when(self.open, |this| this.child(menu))
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

    #[test]
    fn dropdown_menu_auto_dismisses_by_default() {
        let menu = DropdownMenu::new("dropdown", div(), vec![]);

        assert!(menu.auto_dismiss);
    }
}
