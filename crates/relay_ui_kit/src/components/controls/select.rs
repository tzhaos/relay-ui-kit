use std::rc::Rc;

use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    components::overlay::{Menu, MenuItem, overlay},
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, radius},
};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type SelectHandler = Box<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;

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
    on_toggle: Option<ClickHandler>,
    on_select: Option<SelectHandler>,
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
            on_toggle: None,
            on_select: None,
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

    pub fn selected_label(&self) -> &str {
        self.options
            .iter()
            .find(|option| option.key == self.selected_key)
            .map_or(self.placeholder.as_str(), |option| option.label.as_str())
    }
}

impl RenderOnce for Select {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let label = self.selected_label().to_string();
        let selected_key = self.selected_key;
        let select_handler = self.on_select.map(Rc::new);
        let mut root = div().id(self.id).relative().flex().items_center().child(
            div()
                .id("select-trigger")
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
                .cursor_pointer()
                .hover(move |style| style.bg(theme.hover).border_color(theme.border_strong))
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
                .when_some(self.on_toggle, |this, handler| {
                    this.on_click(move |event, window, cx| {
                        handler(event, window, cx);
                        cx.stop_propagation();
                    })
                }),
        );

        if self.open {
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
                if let Some(handler) = select_handler.clone() {
                    item = item.on_click(move |_event, window, cx| handler(key, window, cx));
                }
                items.push(item);
            }
            root = root
                .child(overlay(Menu::new("select-menu", items).min_width(220.0)).offset(0.0, 34.0));
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_uses_selected_option_label() {
        let select = Select::new("select", "dark", vec![SelectOption::new("dark", "Dark")]);

        assert_eq!(select.selected_label(), "Dark");
    }
}
