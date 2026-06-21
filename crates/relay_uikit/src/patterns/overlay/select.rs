use std::rc::Rc;

use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{ClickHandler, DismissHandler, SelectHandler},
    theme::{ActiveTheme, radius},
};

use super::{AnchoredOverlay, Menu, MenuItem};

/// One option in a [`Select`].
pub struct SelectOption {
    key: &'static str,
    label: String,
    detail: Option<String>,
    icon: Option<IconName>,
}

impl SelectOption {
    pub fn new(key: &'static str, label: impl Into<String>) -> Self {
        Self {
            key,
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
pub struct Select {
    id: ElementId,
    selected_key: &'static str,
    options: Vec<SelectOption>,
    open: bool,
    placeholder: String,
    auto_dismiss: bool,
    binding: Option<Binding<&'static str>>,
    on_toggle: Option<ClickHandler>,
    on_select: Option<SelectHandler>,
    on_dismiss: Option<DismissHandler>,
}

impl Select {
    pub fn new(
        id: impl Into<ElementId>,
        selected_key: &'static str,
        options: Vec<SelectOption>,
    ) -> Self {
        Self {
            id: id.into(),
            selected_key,
            options,
            open: false,
            placeholder: "Select".into(),
            auto_dismiss: true,
            binding: None,
            on_toggle: None,
            on_select: None,
            on_dismiss: None,
        }
    }

    pub fn bound(
        id: impl Into<ElementId>,
        binding: Binding<&'static str>,
        options: Vec<SelectOption>,
    ) -> Self {
        Self {
            id: id.into(),
            selected_key: "",
            options,
            open: false,
            placeholder: "Select".into(),
            auto_dismiss: true,
            binding: Some(binding),
            on_toggle: None,
            on_select: None,
            on_dismiss: None,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
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
        handler: impl Fn(&'static str, &mut Window, &mut App) + 'static,
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
            .find(|option| option.key == self.selected_key)
            .map_or(self.placeholder.as_str(), |option| option.label.as_str())
    }
}

impl RenderOnce for Select {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        debug_assert!(
            !self.options.is_empty(),
            "Select `{}` must have at least one option",
            self.id
        );
        let theme = *cx.theme();
        let binding = self.binding;
        let selected_key = binding
            .as_ref()
            .map_or(self.selected_key, |binding| binding.get(cx));
        let label = selected_label(&self.options, selected_key, &self.placeholder).to_string();
        let select_handler = self.on_select.map(Rc::new);
        let dismiss_handler = self.on_dismiss;
        let auto_dismiss = self.auto_dismiss;
        let id = self.id.clone();
        let toggle_handler = self.on_toggle;
        let trigger_clickable = toggle_handler.is_some();
        let trigger = div()
            .id((id.clone(), "trigger"))
            .h(px(30.0))
            .min_w(px(180.0))
            .max_w(px(280.0))
            .px_2()
            .flex()
            .items_center()
            .gap_2()
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
            .when_some(
                toggle_handler.filter(|_| trigger_clickable),
                |this, handler| {
                    this.on_click(move |event, window, cx| {
                        handler(event, window, cx);
                        cx.stop_propagation();
                    })
                },
            );

        let mut items = Vec::with_capacity(self.options.len());
        for option in self.options {
            let key = option.key;
            let mut item = MenuItem::new(option.label).checked(key == selected_key);
            if let Some(detail) = option.detail {
                item = item.detail(detail);
            }
            if let Some(icon) = option.icon {
                item = item.icon(icon);
            }
            if binding.is_some() || select_handler.is_some() {
                let binding = binding.clone();
                let handler = select_handler.clone();
                item = item.on_click(move |_event, window, cx| {
                    if let Some(binding) = &binding {
                        binding.set(cx, key);
                    }
                    if let Some(handler) = &handler {
                        handler(key, window, cx);
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
        .open(self.open);

        if auto_dismiss {
            if let Some(dismiss_handler) = dismiss_handler {
                overlay = overlay.on_dismiss(dismiss_handler);
            } else if let Some(binding) = binding {
                let binding_clone = binding.clone();
                overlay = overlay.on_dismiss(move |_window, cx| {
                    binding_clone.set(cx, selected_key);
                });
            } else {
                // Auto-dismiss without explicit handler is a no-op visually
                // but ensures the overlay has dismiss wiring
                let id_for_dismiss = id.clone();
                overlay = overlay.on_dismiss(move |_window, _cx| {
                    // no-op: overlay dismissed but no state to update
                    let _ = &id_for_dismiss;
                });
            }
        } else if let Some(dismiss_handler) = dismiss_handler {
            overlay = overlay.on_dismiss(dismiss_handler);
        }

        overlay
    }
}

fn selected_label<'a>(
    options: &'a [SelectOption],
    selected_key: &'static str,
    placeholder: &'a str,
) -> &'a str {
    options
        .iter()
        .find(|option| option.key == selected_key)
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

        assert!(select.binding.is_some());
    }

    #[test]
    fn select_auto_dismiss_defaults_to_true() {
        let select = Select::new("select", "dark", vec![SelectOption::new("dark", "Dark")]);

        assert!(select.auto_dismiss);
    }
}
