mod item;

use gpui::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

pub use item::MenuItem;

use crate::{
    icon::{Icon, IconName, IconSize},
    motion::{MotionDirection, MotionExt},
    theme::{ActiveTheme, BORDER_WIDTH, DISABLED_OPACITY, radius, space},
};

use super::overlay;

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
            .gap(px(BORDER_WIDTH))
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
                panel = panel.child(
                    div()
                        .my(px(space::XS))
                        .h(px(BORDER_WIDTH))
                        .w_full()
                        .bg(theme.border),
                );
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
                .when(disabled, |this| this.opacity(DISABLED_OPACITY))
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
        .gap(px(BORDER_WIDTH))
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
