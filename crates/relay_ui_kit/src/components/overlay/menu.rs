use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::ClickHandler,
    motion::{MotionDirection, MotionExt},
    theme::{ActiveTheme, radius, space},
};

use super::overlay;

/// One row in a [`Menu`].
pub struct MenuItem {
    label: String,
    detail: Option<String>,
    icon: Option<IconName>,
    trailing: Option<AnyElement>,
    checked: bool,
    danger: bool,
    disabled: bool,
    separator: bool,
    header: bool,
    submenu: bool,
    submenu_items: Vec<MenuItem>,
    submenu_open: bool,
    on_click: Option<ClickHandler>,
}

impl MenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            detail: None,
            icon: None,
            trailing: None,
            checked: false,
            danger: false,
            disabled: false,
            separator: false,
            header: false,
            submenu: false,
            submenu_items: Vec::new(),
            submenu_open: false,
            on_click: None,
        }
    }

    /// A 1px divider row between groups of items.
    pub fn separator() -> Self {
        Self {
            label: String::new(),
            detail: None,
            icon: None,
            trailing: None,
            checked: false,
            danger: false,
            disabled: false,
            separator: true,
            header: false,
            submenu: false,
            submenu_items: Vec::new(),
            submenu_open: false,
            on_click: None,
        }
    }

    pub fn header(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            detail: None,
            icon: None,
            trailing: None,
            checked: false,
            danger: false,
            disabled: true,
            separator: false,
            header: true,
            submenu: false,
            submenu_items: Vec::new(),
            submenu_open: false,
            on_click: None,
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

    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Render in the danger tone.
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn submenu(mut self) -> Self {
        self.submenu = true;
        self
    }

    pub fn submenu_items(mut self, items: Vec<MenuItem>) -> Self {
        self.submenu = true;
        self.submenu_items = items;
        self
    }

    pub fn submenu_open(mut self, open: bool) -> Self {
        self.submenu_open = open;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

/// A floating menu panel.
#[derive(IntoElement)]
pub struct Menu {
    id: ElementId,
    items: Vec<MenuItem>,
    min_width: f32,
}

impl Menu {
    pub fn new(id: impl Into<ElementId>, items: Vec<MenuItem>) -> Self {
        Self {
            id: id.into(),
            items,
            min_width: 180.0,
        }
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }
}

impl RenderOnce for Menu {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let mut panel = div()
            .id(self.id)
            .min_w(px(self.min_width))
            .p(px(space::XS))
            .flex()
            .flex_col()
            .gap(px(1.0))
            .rounded(px(radius::LG))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border_strong)
            .shadow_lg()
            .occlude();

        for (index, item) in self.items.into_iter().enumerate() {
            let MenuItem {
                label,
                detail,
                icon,
                trailing,
                checked,
                danger,
                disabled,
                separator,
                header,
                submenu,
                submenu_items,
                submenu_open,
                on_click,
            } = item;

            if separator {
                panel = panel.child(div().my(px(space::XS)).h(px(1.0)).w_full().bg(theme.border));
                continue;
            }
            if header {
                panel = panel.child(menu_header(label, theme.text_muted));
                continue;
            }

            let fg = if danger { theme.danger } else { theme.text };
            let icon_color = if danger {
                theme.danger
            } else {
                theme.text_muted
            };
            let has_submenu = submenu || !submenu_items.is_empty();
            let submenu_visible = submenu_open && !submenu_items.is_empty();
            let row_content = div()
                .id(("menu-item", index))
                .min_h(px(if detail.is_some() { 38.0 } else { 28.0 }))
                .px_2()
                .py_1()
                .flex()
                .items_center()
                .gap_2()
                .rounded(px(radius::MD))
                .text_sm()
                .text_color(fg)
                .when(disabled, |this| this.opacity(0.52))
                .when(!disabled, |this| {
                    this.cursor_pointer().hover(move |s| s.bg(theme.hover))
                })
                .child(menu_leading_icon(checked, icon, icon_color, theme.accent))
                .child(menu_label(label, detail, theme.text_muted))
                .when_some(trailing, |this, trailing| this.child(trailing))
                .when(has_submenu, |this| {
                    this.child(
                        Icon::new(IconName::ChevronRight)
                            .size(IconSize::XSmall)
                            .color(theme.text_muted),
                    )
                })
                .when_some(on_click.filter(|_| !disabled), |this, handler| {
                    this.on_click(move |event, window, cx| {
                        handler(event, window, cx);
                        cx.stop_propagation();
                    })
                });
            let row = div()
                .relative()
                .child(row_content)
                .when(submenu_visible, |this| {
                    this.child(
                        overlay(Menu::new(("submenu", index), submenu_items).min_width(180.0))
                            .offset(self.min_width - 4.0, 0.0),
                    )
                });
            panel = panel.child(row);
        }
        panel.motion_slide_in(MotionDirection::FromTop, true)
    }
}

fn menu_leading_icon(
    checked: bool,
    icon: Option<IconName>,
    icon_color: gpui::Hsla,
    accent: gpui::Hsla,
) -> impl IntoElement {
    div()
        .w(px(16.0))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .when(checked, |this| {
            this.child(
                Icon::new(IconName::Check)
                    .size(IconSize::Small)
                    .color(accent),
            )
        })
        .when(!checked, |this| {
            this.when_some(icon, |this, icon| {
                this.child(Icon::new(icon).size(IconSize::Small).color(icon_color))
            })
        })
}

fn menu_label(label: String, detail: Option<String>, detail_color: gpui::Hsla) -> impl IntoElement {
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(1.0))
        .child(
            div()
                .truncate()
                .font_weight(FontWeight::MEDIUM)
                .child(label),
        )
        .when_some(detail, |this, detail| {
            this.child(
                div()
                    .truncate()
                    .text_size(px(11.0))
                    .text_color(detail_color)
                    .child(detail),
            )
        })
}

fn menu_header(label: String, color: gpui::Hsla) -> impl IntoElement {
    div()
        .min_h(px(24.0))
        .px_2()
        .flex()
        .items_center()
        .text_size(px(11.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(color)
        .child(label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submenu_items_mark_item_as_submenu() {
        let item = MenuItem::new("Open With").submenu_items(vec![MenuItem::new("Shell")]);

        assert!(item.submenu);
    }

    #[test]
    fn menu_header_is_not_interactive() {
        let item = MenuItem::header("Actions");

        assert!(item.header);
        assert!(item.disabled);
    }
}
