use gpui::{App, ElementId, IntoElement, RenderOnce, Window};

use crate::interaction::SharedActionHandler;
use crate::patterns::{Menu, MenuItem};

use super::picker_types::PickerActionKind;

/// Compact menu for item-level management actions such as checkout or rename.
#[derive(IntoElement)]
pub struct ActionsMenu {
    id: ElementId,
    actions: Vec<PickerActionKind>,
    on_select: Option<SharedActionHandler<PickerActionKind>>,
}

impl ActionsMenu {
    /// Create a menu with the standard picker action set.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            actions: vec![
                PickerActionKind::Checkout,
                PickerActionKind::NewWorktree,
                PickerActionKind::Rename,
                PickerActionKind::Delete,
            ],
            on_select: None,
        }
    }

    /// Override the action order and membership shown in the menu.
    pub fn actions(mut self, actions: Vec<PickerActionKind>) -> Self {
        self.actions = actions;
        self
    }

    /// Observe item action selection with the resolved [`PickerActionKind`].
    pub fn on_select(
        mut self,
        handler: impl Fn(PickerActionKind, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(std::rc::Rc::new(handler));
        self
    }
}

impl RenderOnce for ActionsMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let handler = self.on_select;
        let mut items = Vec::with_capacity(self.actions.len() + 1);

        for action in self.actions {
            if action == PickerActionKind::Delete && !items.is_empty() {
                items.push(MenuItem::separator());
            }

            let action_handler = handler.clone();
            let mut item = MenuItem::new(action.label()).icon(action.icon());
            if action.is_dangerous() {
                item = item.danger();
            }
            if let Some(action_handler) = action_handler {
                item = item.on_click(move |_event, window, cx| {
                    action_handler(action, window, cx);
                });
            }
            items.push(item);
        }

        Menu::new(self.id, items)
            .min_width(232.0)
            .render(window, cx)
    }
}
