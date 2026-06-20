use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use relay_ui_primitives::{
    icon::{Icon, IconName, IconSize},
    interaction::{ClickHandler, SharedDismissHandler, SharedSelectHandler},
    theme::{ActiveTheme, radius},
};

use super::panel::{branch_picker_panel, default_picker_actions};
use super::types::{BranchOption, BranchPickerAction};

/// Compact branch selector for title bars and pane toolbars.
#[derive(IntoElement)]
pub struct BranchSelector {
    id: ElementId,
    selected_key: &'static str,
    branches: Vec<BranchOption>,
    actions: Vec<BranchPickerAction>,
    open: bool,
    on_toggle: Option<ClickHandler>,
    on_select: Option<SharedSelectHandler>,
    on_action: Option<SharedSelectHandler>,
    on_dismiss: Option<SharedDismissHandler>,
}

impl BranchSelector {
    pub fn new(
        id: impl Into<ElementId>,
        selected_key: &'static str,
        branches: Vec<BranchOption>,
    ) -> Self {
        Self {
            id: id.into(),
            selected_key,
            branches,
            actions: default_picker_actions(),
            open: false,
            on_toggle: None,
            on_select: None,
            on_action: None,
            on_dismiss: None,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn actions(mut self, actions: Vec<BranchPickerAction>) -> Self {
        self.actions = actions;
        self
    }

    relay_ui_primitives::callback_builder!(on_toggle, on_toggle, ClickEvent);

    relay_ui_primitives::shared_callback_builder!(on_select, on_select, &'static str);

    relay_ui_primitives::shared_callback_builder!(on_action, on_action, &'static str);

    relay_ui_primitives::shared_callback_builder!(on_dismiss, on_dismiss,);

    pub fn selected_label(&self) -> &str {
        self.branches
            .iter()
            .find(|branch| branch.key == self.selected_key)
            .map_or(self.selected_key, |branch| branch.label.as_str())
    }
}

impl RenderOnce for BranchSelector {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let selected_label = self.selected_label().to_string();
        let selected_key = self.selected_key;
        let select_handler = self.on_select;
        let action_handler = self.on_action;
        let dismiss_handler = self.on_dismiss;
        let trigger_handler = self.on_toggle;
        let mut root = div().id(self.id).relative().flex().items_center().child(
            div()
                .id("branch-selector-trigger")
                .h(px(28.0))
                .max_w(px(260.0))
                .px_2()
                .flex()
                .items_center()
                .gap_1()
                .rounded(px(radius::MD))
                .border_1()
                .border_color(if self.open {
                    theme.border_strong
                } else {
                    theme.border
                })
                .bg(if self.open {
                    theme.panel_alt
                } else {
                    theme.panel
                })
                .text_color(theme.text_secondary)
                .cursor_pointer()
                .hover(move |style| style.bg(theme.hover).border_color(theme.border_strong))
                .child(
                    Icon::new(IconName::GitBranch)
                        .size(IconSize::Small)
                        .color(theme.text_muted),
                )
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .child(selected_label),
                )
                .child(
                    Icon::new(IconName::ChevronDown)
                        .size(IconSize::XSmall)
                        .color(theme.text_muted),
                )
                .when_some(trigger_handler, |this, handler| {
                    this.on_click(move |event, window, cx| {
                        handler(event, window, cx);
                        cx.stop_propagation();
                    })
                }),
        );

        if self.open {
            root = root.child(branch_picker_panel(
                selected_key,
                self.branches,
                self.actions,
                select_handler,
                action_handler,
                dismiss_handler,
            ));
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_selector_uses_branch_label_for_selected_key() {
        let selector = BranchSelector::new(
            "branch-selector",
            "feat-ui",
            vec![BranchOption::new("feat-ui", "feature/ui-kit")],
        );

        assert_eq!(selector.selected_label(), "feature/ui-kit");
    }

    #[test]
    fn branch_selector_falls_back_to_selected_key() {
        let selector = BranchSelector::new("branch-selector", "main", vec![]);

        assert_eq!(selector.selected_label(), "main");
    }
}
