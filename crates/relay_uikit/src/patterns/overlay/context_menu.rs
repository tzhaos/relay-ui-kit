use gpui::{Anchor, App, ElementId, IntoElement, RenderOnce, Window, div};
use relay::Binding;

use crate::interaction::{DismissHandler, OpenState};

use super::{Menu, MenuItem, overlay};

/// A right-click or more-actions menu placed relative to its anchor.
#[derive(IntoElement)]
pub struct ContextMenu {
    id: ElementId,
    items: Vec<MenuItem>,
    open: bool,
    min_width: f32,
    left: f32,
    top: f32,
    anchor: Anchor,
    auto_dismiss: bool,
    open_state: Option<OpenState>,
    on_dismiss: Option<DismissHandler>,
}

impl ContextMenu {
    pub fn new(id: impl Into<ElementId>, items: Vec<MenuItem>) -> Self {
        Self {
            id: id.into(),
            items,
            open: false,
            min_width: 180.0,
            left: 0.0,
            top: 0.0,
            anchor: Anchor::TopLeft,
            auto_dismiss: true,
            open_state: None,
            on_dismiss: None,
        }
    }

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

    pub fn offset(mut self, left: f32, top: f32) -> Self {
        self.left = left;
        self.top = top;
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn auto_dismiss(mut self, auto_dismiss: bool) -> Self {
        self.auto_dismiss = auto_dismiss;
        self
    }

    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ContextMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let open_state = self.open_state;
        let open = open_state.as_ref().map_or(self.open, |state| state.get(cx));
        if !open {
            return div().into_any_element();
        }

        let menu_focus = window
            .use_keyed_state((self.id.clone(), "menu-focus"), cx, |_, cx| {
                cx.focus_handle()
            })
            .read(cx)
            .clone();
        let dismiss_handler = self.on_dismiss.map(std::rc::Rc::new);
        let mut menu = Menu::new(self.id, self.items)
            .min_width(self.min_width)
            .focus_handle(menu_focus.clone());
        if self.auto_dismiss
            && let Some(open_state) = open_state.clone()
        {
            menu = menu.on_action_dismiss(move |_window, cx| {
                open_state.close(cx);
            });
        } else if let Some(handler) = dismiss_handler.clone() {
            menu = menu.on_action_dismiss(move |window, cx| {
                handler(window, cx);
            });
        }
        let mut ov = overlay(menu)
            .anchor(self.anchor)
            .offset(self.left, self.top)
            .focus_handle(menu_focus);

        if self.auto_dismiss {
            let open_state = open_state.clone();
            if let Some(handler) = dismiss_handler {
                ov = ov.on_dismiss(move |window, cx| {
                    if let Some(open_state) = &open_state {
                        open_state.close(cx);
                    }
                    handler(window, cx);
                });
            } else {
                ov = ov.on_dismiss(move |_window, cx| {
                    if let Some(open_state) = &open_state {
                        open_state.close(cx);
                    }
                });
            }
        } else if let Some(handler) = dismiss_handler {
            ov = ov.on_dismiss(move |window, cx| {
                handler(window, cx);
            });
        } else {
            // Always wire dismiss so overlay closes on click-outside / Escape
            ov = ov.on_dismiss(|_window, _cx| {});
        }

        ov.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use relay::ReactiveAppExt;

    use super::*;

    #[test]
    fn context_menu_defaults_to_closed() {
        let menu = ContextMenu::new("context", vec![]);

        assert!(!menu.open);
    }

    #[test]
    fn context_menu_default_offset_is_zero() {
        let menu = ContextMenu::new("context", vec![]);

        assert_eq!(menu.left, 0.0);
    }

    #[test]
    fn open_bound_context_menu_stores_open_state() {
        let mut app = gpui::TestApp::new();
        let menu =
            app.update(|cx| ContextMenu::new("context", vec![]).open_bound(cx.binding(false)));

        assert!(menu.open_state.is_some());
    }

    #[test]
    fn context_menu_auto_dismisses_by_default() {
        let menu = ContextMenu::new("context", vec![]);

        assert!(menu.auto_dismiss);
    }
}
