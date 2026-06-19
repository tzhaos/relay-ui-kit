use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    motion::{MotionDirection, MotionExt},
    theme::{ActiveTheme, radius, space},
};

type MenuClick = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

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
    submenu: bool,
    on_click: Option<MenuClick>,
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
            submenu: false,
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
            submenu: false,
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
            if item.separator {
                panel = panel.child(div().my(px(space::XS)).h(px(1.0)).w_full().bg(theme.border));
                continue;
            }

            let fg = if item.danger {
                theme.danger
            } else {
                theme.text
            };
            let icon_color = if item.danger {
                theme.danger
            } else {
                theme.text_muted
            };
            let disabled = item.disabled;
            let row = div()
                .id(("menu-item", index))
                .min_h(px(if item.detail.is_some() { 38.0 } else { 28.0 }))
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
                .child(menu_leading_icon(
                    item.checked,
                    item.icon,
                    icon_color,
                    theme.accent,
                ))
                .child(menu_label(item.label, item.detail, theme.text_muted))
                .when_some(item.trailing, |this, trailing| this.child(trailing))
                .when(item.submenu, |this| {
                    this.child(
                        Icon::new(IconName::ChevronRight)
                            .size(IconSize::XSmall)
                            .color(theme.text_muted),
                    )
                })
                .when_some(item.on_click.filter(|_| !disabled), |this, handler| {
                    this.on_click(move |event, window, cx| {
                        handler(event, window, cx);
                        cx.stop_propagation();
                    })
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
