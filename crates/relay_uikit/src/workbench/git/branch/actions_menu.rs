use gpui::{App, ElementId, IntoElement, RenderOnce, Window};

use crate::interaction::SharedActionHandler;
use crate::patterns::{Menu, MenuItem};

use super::types::BranchActionKind;

/// Context menu for branch management actions.
#[derive(IntoElement)]
pub struct BranchActionsMenu {
    id: ElementId,
    actions: Vec<BranchActionKind>,
    on_select: Option<SharedActionHandler<BranchActionKind>>,
}

impl BranchActionsMenu {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            actions: vec![
                BranchActionKind::Checkout,
                BranchActionKind::NewWorktree,
                BranchActionKind::Rename,
                BranchActionKind::Delete,
            ],
            on_select: None,
        }
    }

    pub fn actions(mut self, actions: Vec<BranchActionKind>) -> Self {
        self.actions = actions;
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(BranchActionKind, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(std::rc::Rc::new(handler));
        self
    }
}

impl RenderOnce for BranchActionsMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let handler = self.on_select;
        let mut items = Vec::with_capacity(self.actions.len() + 1);

        for action in self.actions {
            if action == BranchActionKind::Delete && !items.is_empty() {
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
