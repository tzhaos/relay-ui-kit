use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::patterns::overlay::AnchoredOverlay;
use crate::{
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
    selected_binding: Option<Binding<&'static str>>,
    open_binding: Option<Binding<bool>>,
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
            selected_binding: None,
            open_binding: None,
            on_toggle: None,
            on_select: None,
            on_action: None,
            on_dismiss: None,
        }
    }

    pub fn bound(
        id: impl Into<ElementId>,
        branches: Vec<BranchOption>,
        selected: Binding<&'static str>,
    ) -> Self {
        Self {
            id: id.into(),
            selected_key: "",
            branches,
            actions: default_picker_actions(),
            open: false,
            selected_binding: Some(selected),
            open_binding: None,
            on_toggle: None,
            on_select: None,
            on_action: None,
            on_dismiss: None,
        }
    }

    pub fn open_bound(mut self, binding: Binding<bool>) -> Self {
        self.open_binding = Some(binding);
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn actions(mut self, actions: Vec<BranchPickerAction>) -> Self {
        self.actions = actions;
        self
    }

    pub fn on_toggle(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&'static str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(std::rc::Rc::new(handler));
        self
    }

    pub fn on_action(
        mut self,
        handler: impl Fn(&'static str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_action = Some(std::rc::Rc::new(handler));
        self
    }

    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(std::rc::Rc::new(handler));
        self
    }

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
        let selected_binding = self.selected_binding;
        let open_binding = self.open_binding;
        let selected_key = selected_binding.as_ref().map_or(self.selected_key, |b| b.get(cx));
        let open = open_binding.as_ref().map_or(self.open, |b| b.get(cx));
        let selected_label = self
            .branches
            .iter()
            .find(|branch| branch.key == selected_key)
            .map_or(selected_key, |branch| branch.label.as_str())
            .to_string();
        let select_handler = self.on_select;
        let action_handler = self.on_action;
        let dismiss_handler = self.on_dismiss;
        let trigger_handler = self.on_toggle;
        let trigger_clickable = open_binding.is_some() || trigger_handler.is_some();
        let trigger = div()
            .id("branch-selector-trigger")
            .h(px(28.0))
            .max_w(px(260.0))
            .px_2()
            .flex()
            .items_center()
            .gap_1()
            .rounded(px(radius::MD))
            .border_1()
            .border_color(if open {
                theme.border_strong
            } else {
                theme.border
            })
            .bg(if open {
                theme.panel_alt
            } else {
                theme.panel
            })
            .text_color(theme.text_secondary)
            .when(trigger_clickable, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(theme.hover).border_color(theme.border_strong))
                    .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                        window.prevent_default();
                    })
            })
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
            .when(trigger_clickable, |this| {
                let open_binding = open_binding.clone();
                this.on_click(move |event, window, cx| {
                    if let Some(binding) = &open_binding {
                        binding.update(cx, |open| {
                            *open = !*open;
                            true
                        });
                    }
                    if let Some(handler) = &trigger_handler {
                        handler(event, window, cx);
                    }
                    cx.stop_propagation();
                })
            });

        let mut overlay = AnchoredOverlay::new(
            self.id,
            trigger,
            branch_picker_panel(
                selected_key,
                self.branches,
                self.actions,
                selected_binding.clone(),
                select_handler,
                action_handler,
            ),
        )
        .open(open);

        if let Some(dismiss_handler) = dismiss_handler {
            overlay = overlay.on_dismiss(move |window, cx| dismiss_handler(window, cx));
        } else {
            let dismiss_binding = open_binding.clone();
            overlay = overlay.on_dismiss(move |_window, cx| {
                if let Some(binding) = &dismiss_binding {
                    binding.set(cx, false);
                }
            });
        }

        overlay
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
