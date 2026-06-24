use gpui::{
    AnyElement, ElementId, FocusHandle, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Role, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    motion::{MotionDirection, MotionExt},
    theme::{BORDER_WIDTH, Theme, radius, space},
};

pub(super) struct DialogPanel {
    pub theme: Theme,
    pub id: ElementId,
    pub title: String,
    pub description: Option<String>,
    pub icon: Option<IconName>,
    pub width: f32,
    pub focus_handle: FocusHandle,
    pub children: Vec<AnyElement>,
    pub footer: Option<AnyElement>,
}

impl DialogPanel {
    pub fn render(self) -> impl IntoElement {
        let theme = self.theme;
        div()
            .id(self.id)
            .w(px(self.width))
            .max_w_full()
            .role(Role::Dialog)
            .aria_label(self.title.clone())
            .tab_index(0)
            .track_focus(&self.focus_handle)
            .rounded(px(radius::LG))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border_strong)
            .shadow_lg()
            .overflow_hidden()
            .occlude()
            .flex()
            .flex_col()
            .child(dialog_header(
                theme,
                self.title,
                self.description,
                self.icon,
            ))
            .when(!self.children.is_empty(), |this| {
                this.child(
                    div()
                        .px(px(space::LG))
                        .pb(px(space::LG))
                        .flex()
                        .flex_col()
                        .gap_2()
                        .children(self.children),
                )
            })
            .when_some(self.footer, |this, footer| {
                this.child(
                    div()
                        .p(px(space::LG))
                        .border_t_1()
                        .border_color(theme.border)
                        .bg(theme.panel_alt)
                        .child(footer),
                )
            })
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            })
            .on_click(|_, _, cx| cx.stop_propagation())
            .motion_slide_in(MotionDirection::FromBottom, true)
    }
}

fn dialog_header(
    theme: Theme,
    title: String,
    description: Option<String>,
    icon: Option<IconName>,
) -> gpui::Div {
    div()
        .p(px(space::LG))
        .flex()
        .items_start()
        .gap_3()
        .when_some(icon, |this, icon| {
            this.child(
                div()
                    .mt(px(BORDER_WIDTH))
                    .size(px(28.0))
                    .rounded(px(radius::MD))
                    .bg(theme.panel_alt)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        Icon::new(icon)
                            .size(IconSize::Small)
                            .color(theme.text_secondary),
                    ),
            )
        })
        .child(
            div()
                .min_w_0()
                .flex_1()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text)
                        .child(title),
                )
                .when_some(description, |this, description| {
                    this.child(
                        div()
                            .text_sm()
                            .line_height(px(18.0))
                            .text_color(theme.text_secondary)
                            .child(description),
                    )
                }),
        )
}
