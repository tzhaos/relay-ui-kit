use std::{hash::Hash, rc::Rc};

use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, RenderOnce, Role, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::{Binding, WindowSignalExt};

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{
        ActionHandler, ClickHandler, DismissHandler, OpenState, SelectionSource, SharedClickHandler,
    },
    theme::{ActiveTheme, DISABLED_OPACITY, radius},
};

use super::{AnchoredOverlay, Menu, MenuItem};

/// One option in a [`Select`].
pub struct SelectOption<T> {
    value: T,
    label: String,
    detail: Option<String>,
    icon: Option<IconName>,
}

impl<T> SelectOption<T> {
    /// Create an option with a stable value and visible label.
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            detail: None,
            icon: None,
        }
    }

    /// Add supporting detail text inside the menu row.
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Add a leading icon inside the menu row.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// A compact select trigger plus optional dropdown menu.
///
/// `Select` keeps selection and open-state ownership explicit. Use
/// [`Select::bound`] when the chosen value belongs to Relay or host state, and
/// [`Select::open_bound`] when the surrounding surface wants to coordinate
/// overlay lifetime directly.
#[derive(IntoElement)]
pub struct Select<T>
where
    T: Clone + Eq + Hash + PartialEq + 'static,
{
    id: ElementId,
    selected_value: Option<T>,
    options: Vec<SelectOption<T>>,
    open: bool,
    disabled: bool,
    placeholder: String,
    auto_dismiss: bool,
    source: Option<SelectionSource<T>>,
    open_state: Option<OpenState>,
    aria_label: Option<String>,
    on_toggle: Option<ClickHandler>,
    on_select: Option<ActionHandler<T>>,
    on_dismiss: Option<DismissHandler>,
}

impl<T> Select<T>
where
    T: Clone + Eq + Hash + PartialEq + 'static,
{
    /// Create a select from a host-owned selected value and option list.
    pub fn new(id: impl Into<ElementId>, selected_value: T, options: Vec<SelectOption<T>>) -> Self {
        Self {
            id: id.into(),
            selected_value: Some(selected_value),
            options,
            open: false,
            disabled: false,
            placeholder: "Select".into(),
            auto_dismiss: true,
            source: None,
            open_state: None,
            aria_label: None,
            on_toggle: None,
            on_select: None,
            on_dismiss: None,
        }
    }

    /// Create a select whose chosen value is driven by a [`Binding<T>`].
    pub fn bound(
        id: impl Into<ElementId>,
        binding: Binding<T>,
        options: Vec<SelectOption<T>>,
    ) -> Self {
        Self {
            id: id.into(),
            selected_value: None,
            options,
            open: false,
            disabled: false,
            placeholder: "Select".into(),
            auto_dismiss: true,
            source: Some(SelectionSource::binding(binding)),
            open_state: None,
            aria_label: None,
            on_toggle: None,
            on_select: None,
            on_dismiss: None,
        }
    }

    /// Render the dropdown open or closed from a host-owned snapshot.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Disable trigger activation and menu interaction.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Bind overlay lifetime to shared Relay/host open state.
    pub fn open_bound(mut self, binding: Binding<bool>) -> Self {
        self.open_state = Some(OpenState::binding(binding));
        self
    }

    /// Override the trigger label when nothing is selected.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Override the accessible name for the combobox trigger.
    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// Control whether dismissing the menu also closes the shared open state.
    pub fn auto_dismiss(mut self, auto_dismiss: bool) -> Self {
        self.auto_dismiss = auto_dismiss;
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

    /// Observe option selection after shared selection/open-state updates run.
    pub fn on_select(mut self, handler: impl Fn(T, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }

    /// Observe overlay dismissal after shared open-state cleanup runs.
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }

    pub fn selected_label(&self) -> &str {
        self.options
            .iter()
            .find(|option| {
                self.selected_value
                    .as_ref()
                    .is_some_and(|selected| selected == &option.value)
            })
            .map_or(self.placeholder.as_str(), |option| option.label.as_str())
    }
}

impl<T> RenderOnce for Select<T>
where
    T: Clone + Eq + Hash + PartialEq + 'static,
{
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        debug_assert!(
            !self.options.is_empty(),
            "Select `{}` must have at least one option",
            self.id
        );
        let theme = *cx.theme();
        let Self {
            id,
            selected_value,
            options,
            open,
            disabled,
            placeholder,
            auto_dismiss,
            source,
            open_state,
            aria_label,
            on_toggle,
            on_select,
            on_dismiss,
        } = self;
        let source = source.or_else(|| {
            selected_value.map(|selected_value| {
                SelectionSource::binding(window.use_binding(
                    (id.clone(), "selected-value"),
                    cx,
                    || selected_value,
                ))
            })
        });
        let open_state = open_state.or_else(|| {
            Some(OpenState::binding(window.use_binding(
                (id.clone(), "open-state"),
                cx,
                || open,
            )))
        });
        let trigger_focus = window
            .use_keyed_state((id.clone(), "trigger-focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let menu_focus = window
            .use_keyed_state((id.clone(), "menu-focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let selected_value = source.as_ref().and_then(|source| source.get(cx));
        let open = open_state
            .as_ref()
            .is_some_and(|open_state| open_state.get(cx));
        let label = selected_label(&options, selected_value.as_ref(), &placeholder).to_string();
        let select_handler = on_select.map(Rc::new);
        let dismiss_handler = on_dismiss;
        let toggle_handler: Option<SharedClickHandler> = on_toggle.map(Rc::from);
        let can_select = source.is_some() || select_handler.is_some();
        let trigger_clickable = !disabled && (open_state.is_some() || toggle_handler.is_some());
        let aria_label = aria_label.unwrap_or_else(|| format!("{}: {}", placeholder, label));
        let trigger = div()
            .id((id.clone(), "trigger"))
            .h(px(30.0))
            .min_w(px(180.0))
            .max_w(px(280.0))
            .px_2()
            .flex()
            .items_center()
            .gap_2()
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
                theme.text
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
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .truncate()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .child(label),
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
                        "down" => {
                            if let Some(open_state) = &open_for_key {
                                open_state.set(cx, true);
                                cx.stop_propagation();
                            }
                        }
                        "up" => {
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

        let mut items = Vec::with_capacity(options.len());
        for option in options {
            let value = option.value;
            let mut item = MenuItem::new(option.label).checked(
                selected_value
                    .as_ref()
                    .is_some_and(|selected| selected == &value),
            );
            if let Some(detail) = option.detail {
                item = item.detail(detail);
            }
            if let Some(icon) = option.icon {
                item = item.icon(icon);
            }
            if can_select {
                let source = source.clone();
                let open_state = open_state.clone();
                let handler = select_handler.clone();
                item = item.on_click(move |_event, window, cx| {
                    if let Some(source) = &source {
                        source.select(cx, value.clone());
                    }
                    if let Some(open_state) = &open_state {
                        open_state.close(cx);
                    }
                    if let Some(handler) = &handler {
                        handler(value.clone(), window, cx);
                    }
                });
            }
            items.push(item);
        }

        let mut overlay = AnchoredOverlay::new(
            id.clone(),
            trigger,
            Menu::new((id.clone(), "menu"), items)
                .min_width(220.0)
                .focus_handle(menu_focus.clone()),
        )
        .open(open)
        .focus_handle(menu_focus);

        if auto_dismiss {
            let open_state = open_state.clone();
            match dismiss_handler {
                Some(dismiss_handler) => {
                    overlay = overlay.on_dismiss(move |window, cx| {
                        if let Some(open_state) = &open_state {
                            open_state.close(cx);
                        }
                        dismiss_handler(window, cx);
                    });
                }
                None => {
                    overlay = overlay.on_dismiss(move |_window, cx| {
                        if let Some(open_state) = &open_state {
                            open_state.close(cx);
                        }
                    });
                }
            }
        } else if let Some(dismiss_handler) = dismiss_handler {
            overlay = overlay.on_dismiss(dismiss_handler);
        }

        overlay
    }
}

fn selected_label<'a, T>(
    options: &'a [SelectOption<T>],
    selected_value: Option<&T>,
    placeholder: &'a str,
) -> &'a str
where
    T: Eq + Hash + PartialEq,
{
    options
        .iter()
        .find(|option| selected_value.is_some_and(|selected| selected == &option.value))
        .map_or(placeholder, |option| option.label.as_str())
}

#[cfg(test)]
mod tests {
    use relay::ReactiveAppExt;

    use super::*;

    #[test]
    fn select_uses_selected_option_label() {
        let select = Select::new("select", "dark", vec![SelectOption::new("dark", "Dark")]);

        assert_eq!(select.selected_label(), "Dark");
    }

    #[test]
    fn bound_select_stores_binding() {
        let mut app = gpui::TestApp::new();
        let select = app.update(|cx| {
            Select::bound(
                "select",
                cx.binding("dark"),
                vec![SelectOption::new("dark", "Dark")],
            )
        });

        assert!(select.source.is_some());
    }

    #[test]
    fn select_starts_enabled() {
        let select = Select::new("select", "dark", vec![SelectOption::new("dark", "Dark")]);

        assert!(!select.disabled);
    }

    #[test]
    fn open_bound_select_stores_open_state() {
        let mut app = gpui::TestApp::new();
        let select = app.update(|cx| {
            Select::new("select", "dark", vec![SelectOption::new("dark", "Dark")])
                .open_bound(cx.binding(false))
        });

        assert!(select.open_state.is_some());
    }

    #[test]
    fn select_auto_dismiss_defaults_to_true() {
        let select = Select::new("select", "dark", vec![SelectOption::new("dark", "Dark")]);

        assert!(select.auto_dismiss);
    }
}
