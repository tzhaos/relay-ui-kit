use std::hash::Hash;

use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, RenderOnce, Role, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::{Binding, Selector};

use crate::patterns::overlay::AnchoredOverlay;
use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{
        ClickHandler, OpenState, SelectionSource, SharedActionHandler, SharedClickHandler,
        SharedDismissHandler,
    },
    theme::{ActiveTheme, DISABLED_OPACITY, radius},
};

use super::picker_panel::{PickerPanelProps, picker_panel};
use super::picker_types::{PickerAction, PickerOption};

/// Compact item picker for toolbars, panes, and other dense product surfaces.
///
/// `ItemPicker` is the product-facing picker trigger for "select one item, then
/// maybe run a secondary action on that item". Like [`crate::Select`], it keeps
/// selection and open-state ownership explicit. Use [`ItemPicker::title`] and
/// [`ItemPicker::icon`] when a product surface wants domain-specific
/// presentation such as a branch switcher or project chooser without baking
/// that vocabulary into the base primitive.
#[derive(IntoElement)]
pub struct ItemPicker<K>
where
    K: Clone + Eq + Hash + PartialEq + ToString + 'static,
{
    id: ElementId,
    title: String,
    icon: Option<IconName>,
    selected_key: Option<K>,
    items: Vec<PickerOption<K>>,
    actions: Vec<PickerAction>,
    open: bool,
    disabled: bool,
    auto_dismiss: bool,
    selection: Option<SelectionSource<K>>,
    open_state: Option<OpenState>,
    aria_label: Option<String>,
    on_toggle: Option<ClickHandler>,
    on_select: Option<SharedActionHandler<K>>,
    on_action: Option<SharedActionHandler<String>>,
    on_dismiss: Option<SharedDismissHandler>,
}

impl<K> ItemPicker<K>
where
    K: Clone + Eq + Hash + PartialEq + ToString + 'static,
{
    /// Create a picker from a host-owned selected key and option list.
    pub fn new(id: impl Into<ElementId>, selected_key: K, items: Vec<PickerOption<K>>) -> Self {
        Self {
            id: id.into(),
            title: "Select item".into(),
            icon: None,
            selected_key: Some(selected_key),
            items,
            actions: Vec::new(),
            open: false,
            disabled: false,
            auto_dismiss: true,
            selection: None,
            open_state: None,
            aria_label: None,
            on_toggle: None,
            on_select: None,
            on_action: None,
            on_dismiss: None,
        }
    }

    /// Create a picker whose selected key is driven by a [`Binding<K>`].
    pub fn bound(
        id: impl Into<ElementId>,
        items: Vec<PickerOption<K>>,
        selected: Binding<K>,
    ) -> Self {
        Self {
            id: id.into(),
            title: "Select item".into(),
            icon: None,
            selected_key: None,
            items,
            actions: Vec::new(),
            open: false,
            disabled: false,
            auto_dismiss: true,
            selection: Some(SelectionSource::binding(selected)),
            open_state: None,
            aria_label: None,
            on_toggle: None,
            on_select: None,
            on_action: None,
            on_dismiss: None,
        }
    }

    /// Bind overlay lifetime to shared Relay/host open state.
    pub fn open_bound(mut self, binding: Binding<bool>) -> Self {
        self.open_state = Some(OpenState::binding(binding));
        self
    }

    /// Render the picker open or closed from a host-owned snapshot.
    ///
    /// This does not create internal ownership. Pair it with
    /// [`ItemPicker::open_bound`] when the trigger should control panel
    /// visibility directly.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Override the panel title shown above the item list.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Add a leading icon to the trigger and panel header.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Disable trigger activation and panel interaction.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Control whether item selection and action execution close the picker.
    pub fn auto_dismiss(mut self, auto_dismiss: bool) -> Self {
        self.auto_dismiss = auto_dismiss;
        self
    }

    /// Drive selection from a shared selection adapter.
    pub fn selected_with(mut self, selection: SelectionSource<K>) -> Self {
        self.selection = Some(selection);
        self
    }

    /// Drive selection from a plain [`Selector<K>`].
    pub fn selected_by(self, selector: Selector<K>) -> Self {
        self.selected_with(SelectionSource::selector(selector))
    }

    /// Override the optional action rows shown under the item list.
    ///
    /// Actions default to empty so generic item pickers do not inherit any
    /// domain-specific behavior unless the host opts in.
    pub fn actions(mut self, actions: Vec<PickerAction>) -> Self {
        self.actions = actions;
        self
    }

    /// Override the accessible name for the combobox trigger.
    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// Observe trigger activation after shared open-state toggling runs.
    pub fn on_toggle(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    /// Observe item selection after shared selection and optional dismiss cleanup run.
    pub fn on_select(mut self, handler: impl Fn(K, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(std::rc::Rc::new(handler));
        self
    }

    /// Observe secondary action selection after optional dismiss cleanup runs.
    pub fn on_action(mut self, handler: impl Fn(String, &mut Window, &mut App) + 'static) -> Self {
        self.on_action = Some(std::rc::Rc::new(handler));
        self
    }

    /// Observe panel dismissal after shared open-state cleanup runs.
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(std::rc::Rc::new(handler));
        self
    }

    /// Returns the display label for the currently selected item.
    /// Reads from a selector or binding first, falls back to selected_key.
    pub fn selected_label(&self, cx: &App) -> String {
        let Some(key) = self.current_selected_key(cx) else {
            return String::new();
        };

        self.items
            .iter()
            .find(|option| option.key == key)
            .map_or_else(|| key.to_string(), |option| option.label.to_string())
    }

    fn current_selected_key(&self, cx: &App) -> Option<K> {
        self.selection
            .as_ref()
            .and_then(|selection| selection.get(cx))
            .or_else(|| self.selected_key.clone())
    }
}

impl<K> RenderOnce for ItemPicker<K>
where
    K: Clone + Eq + Hash + PartialEq + ToString + 'static,
{
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let Self {
            id,
            title,
            icon,
            selected_key,
            items,
            actions,
            open,
            disabled,
            auto_dismiss,
            selection,
            open_state,
            aria_label,
            on_toggle,
            on_select,
            on_action,
            on_dismiss,
        } = self;
        let selection = selection;
        let open_state = open_state;
        let trigger_focus = window
            .use_keyed_state((id.clone(), "trigger-focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let panel_focus = window
            .use_keyed_state((id.clone(), "panel-focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let selected_key = selection
            .as_ref()
            .and_then(|selection| selection.get(cx))
            .or(selected_key);
        let open = open_state
            .as_ref()
            .map_or(open, |open_state| open_state.get(cx));
        let selected_option = selected_key
            .as_ref()
            .and_then(|selected_key| items.iter().find(|option| option.key == *selected_key));
        let selected_label = selected_key
            .as_ref()
            .map_or_else(String::new, |selected_key| {
                selected_option
                    .map_or_else(|| selected_key.to_string(), |option| option.label.clone())
            });
        let trigger_icon = selected_option.and_then(|option| option.icon).or(icon);
        let select_handler = if auto_dismiss || on_select.is_some() {
            let open_state = open_state.clone();
            Some(
                std::rc::Rc::new(move |key: K, window: &mut Window, cx: &mut App| {
                    if auto_dismiss && let Some(open_state) = &open_state {
                        open_state.close(cx);
                    }
                    if let Some(handler) = &on_select {
                        handler(key, window, cx);
                    }
                }) as SharedActionHandler<K>,
            )
        } else {
            None
        };
        let action_handler = on_action.map(|handler| {
            let open_state = open_state.clone();
            std::rc::Rc::new(move |key: String, window: &mut Window, cx: &mut App| {
                if auto_dismiss && let Some(open_state) = &open_state {
                    open_state.close(cx);
                }
                handler(key, window, cx);
            }) as SharedActionHandler<String>
        });
        let dismiss_handler = on_dismiss;
        let toggle_handler: Option<SharedClickHandler> = on_toggle.map(std::rc::Rc::from);
        let trigger_clickable = !disabled && open_state.is_some();
        let aria_label = aria_label.unwrap_or_else(|| format!("Select item: {selected_label}"));
        let trigger = div()
            .id((id.clone(), "trigger"))
            .h(px(28.0))
            .max_w(px(260.0))
            .px_2()
            .flex()
            .items_center()
            .gap_1()
            .role(Role::ComboBox)
            .aria_expanded(open)
            .aria_label(aria_label)
            .when(!disabled, |this| this.tab_index(0))
            .when(!disabled, |this| this.track_focus(&trigger_focus))
            .rounded(px(radius::MD))
            .border_1()
            .border_color(if open {
                theme.border_strong
            } else {
                theme.border
            })
            .bg(if open { theme.panel_alt } else { theme.panel })
            .text_color(if disabled {
                theme.text_muted.opacity(0.55)
            } else {
                theme.text_secondary
            })
            .when(disabled, |this| {
                this.opacity(DISABLED_OPACITY)
                    .cursor(gpui::CursorStyle::OperationNotAllowed)
            })
            .when(trigger_clickable, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(theme.hover).border_color(theme.border_strong))
                    .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                        window.prevent_default();
                    })
            })
            .when_some(trigger_icon, |this, icon| {
                this.child(
                    Icon::new(icon)
                        .size(IconSize::Small)
                        .color(theme.text_muted),
                )
            })
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
                let open_for_click = open_state.clone();
                let open_for_key = open_state.clone();
                let toggle_for_click = toggle_handler.clone();
                let toggle_for_key = toggle_for_click.clone();
                this.on_click(move |event, window, cx| {
                    if let Some(open_state) = &open_for_click {
                        open_state.toggle(cx);
                    }
                    if let Some(handler) = &toggle_for_click {
                        handler(event, window, cx);
                    }
                    cx.stop_propagation();
                })
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    match event.keystroke.key.as_str() {
                        "enter" | " " => {
                            if let Some(open_state) = &open_for_key {
                                open_state.toggle(cx);
                            }
                            if let Some(handler) = &toggle_for_key {
                                handler(&ClickEvent::default(), window, cx);
                            }
                            cx.stop_propagation();
                        }
                        "down" | "up" => {
                            if let Some(open_state) = &open_for_key {
                                open_state.set(cx, true);
                                cx.stop_propagation();
                            }
                        }
                        "escape" => {
                            if let Some(open_state) = &open_for_key {
                                open_state.close(cx);
                                cx.stop_propagation();
                            }
                        }
                        _ => {}
                    }
                })
            });

        let mut overlay = AnchoredOverlay::new(
            id.clone(),
            trigger,
            picker_panel(PickerPanelProps {
                id: id.clone(),
                focus_handle: panel_focus.clone(),
                title,
                icon,
                selected_key,
                items,
                actions,
                selection: selection.clone(),
                select_handler,
                action_handler,
            }),
        )
        .open(open)
        .focus_handle(panel_focus);

        let open_state = open_state.clone();
        if let Some(dismiss_handler) = dismiss_handler {
            overlay = overlay.on_dismiss(move |window, cx| {
                if let Some(open_state) = &open_state {
                    open_state.close(cx);
                }
                dismiss_handler(window, cx);
            });
        } else {
            overlay = overlay.on_dismiss(move |_window, cx| {
                if let Some(open_state) = &open_state {
                    open_state.close(cx);
                }
            });
        }

        overlay
    }
}

#[cfg(test)]
mod tests {
    use gpui::TestApp;
    use relay::{ReactiveAppExt, SelectionModel, init};

    use super::*;

    #[test]
    fn item_picker_uses_option_label_for_selected_key() {
        let picker = ItemPicker::new(
            "item-picker",
            "feat-ui",
            vec![PickerOption::new("feat-ui", "feature/ui-kit")],
        );
        let app = TestApp::new();
        let label = app.read(|cx| picker.selected_label(cx));

        assert_eq!(label, "feature/ui-kit");
    }

    #[test]
    fn item_picker_falls_back_to_selected_key_when_option_is_missing() {
        let picker = ItemPicker::new("item-picker", "main", vec![]);
        let app = TestApp::new();
        let label = app.read(|cx| picker.selected_label(cx));

        assert_eq!(label, "main");
    }

    #[test]
    fn item_picker_reads_selected_label_from_selector() {
        let mut app = TestApp::new();
        let selector = app.update(|cx| {
            init(cx);
            cx.selector(Some("feat-ui"))
        });
        let picker = ItemPicker::new(
            "item-picker",
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

    #[test]
    fn item_picker_reads_selected_label_from_selection_source() {
        let mut app = TestApp::new();
        let selection = app.update(|cx| {
            init(cx);
            SelectionModel::new(cx, Some("feat-ui"))
        });
        let picker = ItemPicker::new(
            "item-picker",
            "main",
            vec![
                PickerOption::new("main", "main"),
                PickerOption::new("feat-ui", "feature/ui-kit"),
            ],
        )
        .selected_with(SelectionSource::selection_model(selection.clone()));

        let initial = app.read(|cx| picker.selected_label(cx));
        assert_eq!(initial, "feature/ui-kit");

        app.update(|cx| {
            selection.select(cx, "main");
        });

        let selected = app.read(|cx| picker.selected_label(cx));
        assert_eq!(selected, "main");
    }

    #[test]
    fn item_picker_starts_enabled() {
        let picker = ItemPicker::new("item-picker", "main", vec![]);

        assert!(!picker.disabled);
    }

    #[test]
    fn item_picker_defaults_to_no_open_controller() {
        let picker = ItemPicker::new("item-picker", "main", vec![]);

        assert!(picker.open_state.is_none());
    }

    #[test]
    fn item_picker_defaults_title_to_select_item() {
        let picker = ItemPicker::new("item-picker", "main", vec![]);

        assert_eq!(picker.title, "Select item");
    }

    #[test]
    fn item_picker_title_builder_stores_title() {
        let picker = ItemPicker::new("item-picker", "main", vec![]).title("Switch branch");

        assert_eq!(picker.title, "Switch branch");
    }

    #[test]
    fn item_picker_defaults_to_no_secondary_actions() {
        let picker = ItemPicker::new("item-picker", "main", vec![]);

        assert!(picker.actions.is_empty());
    }

    #[test]
    fn item_picker_icon_builder_stores_icon() {
        let picker = ItemPicker::new("item-picker", "main", vec![]).icon(IconName::GitBranch);

        assert_eq!(picker.icon, Some(IconName::GitBranch));
    }

    #[test]
    fn item_picker_auto_dismisses_by_default() {
        let picker = ItemPicker::new("item-picker", "main", vec![]);

        assert!(picker.auto_dismiss);
    }

    #[test]
    fn item_picker_can_disable_auto_dismiss() {
        let picker = ItemPicker::new("item-picker", "main", vec![]).auto_dismiss(false);

        assert!(!picker.auto_dismiss);
    }

    #[test]
    fn open_bound_item_picker_stores_open_state() {
        let mut app = TestApp::new();
        let picker = app.update(|cx| {
            ItemPicker::new("item-picker", "main", vec![]).open_bound(cx.binding(false))
        });

        assert!(picker.open_state.is_some());
    }
}
