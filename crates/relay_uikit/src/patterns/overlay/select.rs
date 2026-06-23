use std::{hash::Hash, rc::Rc};

use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, RenderOnce, Role, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{
        ActionHandler, ClickHandler, DismissHandler, OpenState, SelectionSource,
    },
    theme::{ActiveTheme, radius},
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
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            detail: None,
            icon: None,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// A compact select trigger plus optional dropdown menu.
#[derive(IntoElement)]
pub struct Select<T>
where
    T: Clone + Eq + Hash + PartialEq + 'static,
{
    id: ElementId,
    selected_value: Option<T>,
    options: Vec<SelectOption<T>>,
    open: bool,
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
    pub fn new(id: impl Into<ElementId>, selected_value: T, options: Vec<SelectOption<T>>) -> Self {
        Self {
            id: id.into(),
            selected_value: Some(selected_value),
            options,
            open: false,
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

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn open_bound(mut self, binding: Binding<bool>) -> Self {
        self.open_state = Some(OpenState::binding(binding));
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    pub fn auto_dismiss(mut self, auto_dismiss: bool) -> Self {
        self.auto_dismiss = auto_dismiss;
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
        handler: impl Fn(T, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }

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
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        debug_assert!(
            !self.options.is_empty(),
            "Select `{}` must have at least one option",
            self.id
        );
        let theme = *cx.theme();
        let source = self.source;
        let open_state = self.open_state;
        let selected_value = source
            .as_ref()
            .and_then(|source| source.get(cx))
            .or(self.selected_value);
        let open = open_state.as_ref().map_or(self.open, |open_state| open_state.get(cx));
        let label =
            selected_label(&self.options, selected_value.as_ref(), &self.placeholder).to_string();
        let select_handler = self.on_select.map(Rc::new);
        let dismiss_handler = self.on_dismiss;
        let auto_dismiss = self.auto_dismiss;
        let id = self.id.clone();
        let toggle_handler: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>> =
            self.on_toggle.map(Rc::from);
        let trigger_clickable = open_state.is_some() || toggle_handler.is_some();
        let aria_label = self
            .aria_label
            .unwrap_or_else(|| format!("{}: {}", self.placeholder, label));
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
            .tab_index(0)
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
            .text_color(theme.text)
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

        let mut items = Vec::with_capacity(self.options.len());
        for option in self.options {
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
            if source.is_some() || select_handler.is_some() || open_state.is_some() {
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
            Menu::new((id.clone(), "menu"), items).min_width(220.0),
        )
        .open(open);

        if auto_dismiss {
            match (dismiss_handler, open_state.clone()) {
                (Some(dismiss_handler), Some(open_state)) => {
                    overlay = overlay.on_dismiss(move |window, cx| {
                        open_state.close(cx);
                        dismiss_handler(window, cx);
                    });
                }
                (Some(dismiss_handler), None) => {
                    overlay = overlay.on_dismiss(dismiss_handler);
                }
                (None, Some(open_state)) => {
                    overlay = overlay.on_dismiss(move |_window, cx| {
                        open_state.close(cx);
                    });
                }
                (None, None) => {
                    // Auto-dismiss without explicit handler is a no-op visually
                    // but ensures the overlay has dismiss wiring
                    let id_for_dismiss = id.clone();
                    overlay = overlay.on_dismiss(move |_window, _cx| {
                        let _ = &id_for_dismiss;
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
    fn select_auto_dismiss_defaults_to_true() {
        let select = Select::new("select", "dark", vec![SelectOption::new("dark", "Dark")]);

        assert!(select.auto_dismiss);
    }
}
