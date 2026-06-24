use gpui::{AnyElement, App, ElementId, IntoElement, Pixels, Point, RenderOnce, Window, point, px};
use relay::Binding;

use crate::interaction::{DismissHandler, OpenState};

use super::{AnchoredOverlay, Menu, MenuItem};

#[derive(IntoElement)]
pub struct DropdownMenu {
    id: ElementId,
    trigger: AnyElement,
    items: Vec<MenuItem>,
    open: bool,
    auto_dismiss: bool,
    min_width: f32,
    offset: Point<Pixels>,
    open_state: Option<OpenState>,
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
            offset: point(px(0.0), px(0.0)),
            open_state: None,
            on_dismiss: None,
        }
    }

    /// Render the dropdown open or closed from a host-owned snapshot.
    ///
    /// This does not create internal ownership. Pair it with
    /// [`DropdownMenu::open_bound`] when the trigger should control menu
    /// visibility directly.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn open_bound(mut self, binding: Binding<bool>) -> Self {
        self.open_state = Some(OpenState::binding(binding));
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

    pub fn offset(mut self, offset: Point<Pixels>) -> Self {
        self.offset = offset;
        self
    }

    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for DropdownMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let menu_focus = window
            .use_keyed_state((self.id.clone(), "menu-focus"), cx, |_, cx| {
                cx.focus_handle()
            })
            .read(cx)
            .clone();
        let menu_id = (self.id.clone(), "menu");
        let mut menu = Menu::new(menu_id, self.items)
            .min_width(self.min_width)
            .focus_handle(menu_focus.clone());
        let open_state = self.open_state;
        let open = open_state.as_ref().map_or(self.open, |state| state.get(cx));
        if self.auto_dismiss
            && let Some(open_state) = open_state.clone()
        {
            menu = menu.on_action_dismiss(move |_window, cx| {
                open_state.close(cx);
            });
        }
        let mut overlay = AnchoredOverlay::new(self.id, self.trigger, menu)
            .open(open)
            .focus_handle(menu_focus)
            .offset(self.offset);

        if self.auto_dismiss {
            let open_state = open_state.clone();
            let user_dismiss = self.on_dismiss;
            overlay = overlay.on_dismiss(move |window, cx| {
                if let Some(open_state) = &open_state {
                    open_state.close(cx);
                }
                if let Some(handler) = &user_dismiss {
                    handler(window, cx);
                }
            });
        }

        overlay
    }
}

#[cfg(test)]
mod tests {
    use gpui::div;
    use relay::ReactiveAppExt;

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

        assert_eq!(menu.offset, point(px(0.0), px(0.0)));
    }

    #[test]
    fn dropdown_menu_defaults_to_no_open_controller() {
        let menu = DropdownMenu::new("dropdown", div(), vec![]);

        assert!(menu.open_state.is_none());
    }

    #[test]
    fn dropdown_menu_open_bound_stores_open_state() {
        let mut app = gpui::TestApp::new();
        let menu = app.update(|cx| {
            DropdownMenu::new("dropdown", div(), vec![]).open_bound(cx.binding(false))
        });

        assert!(menu.open_state.is_some());
    }
}
