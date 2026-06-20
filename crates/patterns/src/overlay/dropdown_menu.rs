use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, KeyDownEvent, ParentElement,
    RenderOnce, Styled, Window, div, prelude::FluentBuilder, px,
};

use relay_ui_core::interaction::DismissHandler;

use super::{Menu, MenuItem};

const DEFAULT_MENU_TOP: f32 = 30.0;

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
            top: DEFAULT_MENU_TOP,
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
        let mut menu = div()
            .absolute()
            .left(px(self.left))
            .top(px(self.top))
            .occlude()
            .child(Menu::new(menu_id, self.items).min_width(self.min_width));

        if self.auto_dismiss
            && let Some(on_dismiss) = self.on_dismiss
        {
            let dismiss_for_key = std::rc::Rc::new(on_dismiss);
            let dismiss_for_mouse = dismiss_for_key.clone();
            menu = menu
                .on_mouse_down_out(move |_event, window, cx| {
                    dismiss_for_mouse(window, cx);
                })
                .key_context("DropdownMenu")
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if event.keystroke.key.as_str() == "escape" {
                        dismiss_for_key(window, cx);
                        cx.stop_propagation();
                    }
                });
        }

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

    #[test]
    fn dropdown_menu_defaults_to_trigger_bottom_edge() {
        let menu = DropdownMenu::new("dropdown", div(), vec![]);

        assert_eq!(menu.top, DEFAULT_MENU_TOP);
        assert_eq!(menu.left, 0.0);
    }
}
