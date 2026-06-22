use gpui::{
    div, prelude::FluentBuilder, px, App, ClickEvent, ElementId, FontWeight, InteractiveElement,
    IntoElement, MouseButton, ParentElement, RenderOnce, StatefulInteractiveElement, Styled,
    Window,
};
use relay::{Binding, Selector};

use crate::patterns::overlay::AnchoredOverlay;
use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{ClickHandler, SharedDismissHandler, SharedSelectHandler},
    theme::{radius, ActiveTheme},
};

use super::picker_panel::{branch_picker_panel, default_picker_actions};
use super::picker_types::{PickerAction, PickerOption};

/// Compact branch selector for title bars and pane toolbars.
#[derive(IntoElement)]
pub struct ItemPicker {
    id: ElementId,
    selected_key: &'static str,
    items: Vec<PickerOption>,
    actions: Vec<PickerAction>,
    open: bool,
    selected_binding: Option<Binding<&'static str>>,
    selection: Option<Selector<&'static str>>,
    open_binding: Option<Binding<bool>>,
    on_toggle: Option<ClickHandler>,
    on_select: Option<SharedSelectHandler>,
    on_action: Option<SharedSelectHandler>,
    on_dismiss: Option<SharedDismissHandler>,
}

impl ItemPicker {
    pub fn new(
        id: impl Into<ElementId>,
        selected_key: &'static str,
        items: Vec<PickerOption>,
    ) -> Self {
        Self {
            id: id.into(),
            selected_key,
            items,
            actions: default_picker_actions(),
            open: false,
            selected_binding: None,
            selection: None,
            open_binding: None,
            on_toggle: None,
            on_select: None,
            on_action: None,
            on_dismiss: None,
        }
    }

    pub fn bound(
        id: impl Into<ElementId>,
        items: Vec<PickerOption>,
        selected: Binding<&'static str>,
    ) -> Self {
        Self {
            id: id.into(),
            selected_key: "",
            items,
            actions: default_picker_actions(),
            open: false,
            selected_binding: Some(selected),
            selection: None,
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

    pub fn selected_by(mut self, selector: Selector<&'static str>) -> Self {
        self.selection = Some(selector);
        self
    }

    pub fn actions(mut self, actions: Vec<PickerAction>) -> Self {
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

    /// Returns the display label for the currently selected branch.
    /// Reads from a selector or binding first, falls back to selected_key.
    pub fn selected_label(&self, cx: &App) -> String {
        let key = self.current_selected_key(cx);
        self.items
            .iter()
            .find(|branch| branch.key == key)
            .map_or(key.to_string(), |branch| branch.label.to_string())
    }

    fn current_selected_key(&self, cx: &App) -> &'static str {
        if let Some(selection) = &self.selection {
            if let Some(key) = selection.get(cx) {
                return key;
            }
        }

        self.selected_binding
            .as_ref()
            .map_or(self.selected_key, |b| b.get(cx))
    }
}

impl RenderOnce for ItemPicker {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let selected_binding = self.selected_binding;
        let selection = self.selection;
        let open_binding = self.open_binding;
        let selected_key = if let Some(selection) = &selection {
            selection.get(cx).unwrap_or(self.selected_key)
        } else {
            selected_binding
                .as_ref()
                .map_or(self.selected_key, |b| b.get(cx))
        };
        let open = open_binding.as_ref().map_or(self.open, |b| b.get(cx));
        let selected_label = self
            .items
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
            .bg(if open { theme.panel_alt } else { theme.panel })
            .text_color(theme.text_secondary)
            .when(trigger_clickable, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(theme.hover).border_color(theme.border_strong))
                    .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                        window.prevent_default();
                    })
            })
            .child(
                Icon::new(IconName::Folder)
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
                self.items,
                self.actions,
                selected_binding.clone(),
                selection.clone(),
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
    use gpui::TestApp;
    use relay::{init, ReactiveAppExt};

    use super::*;

    #[test]
    fn branch_selector_uses_branch_label_for_selected_key() {
        let selector = ItemPicker::new(
            "branch-selector",
            "feat-ui",
            vec![PickerOption::new("feat-ui", "feature/ui-kit")],
        );
        let app = TestApp::new();
        let label = app.read(|cx| selector.selected_label(cx));

        assert_eq!(label, "feature/ui-kit");
    }

    #[test]
    fn branch_selector_falls_back_to_selected_key() {
        let selector = ItemPicker::new("branch-selector", "main", vec![]);
        let app = TestApp::new();
        let label = app.read(|cx| selector.selected_label(cx));

        assert_eq!(label, "main");
    }

    #[test]
    fn branch_selector_reads_selected_label_from_selector() {
        let mut app = TestApp::new();
        let selector = app.update(|cx| {
            init(cx);
            cx.selector(Some("feat-ui"))
        });
        let picker = ItemPicker::new(
            "branch-selector",
            "main",
            vec![
                PickerOption::new("main", "main"),
                PickerOption::new("feat-ui", "feature/ui-kit"),
            ],
        )
        .selected_by(selector.clone());

        let initial = app.read(|cx| picker.selected_label(cx));
        assert_eq!(initial, "feature/ui-kit");

        app.update(|cx| {
            selector.select(cx, "main");
        });

        let selected = app.read(|cx| picker.selected_label(cx));
        assert_eq!(selected, "main");
    }
}
