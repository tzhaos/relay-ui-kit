use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, space},
};

/// A compact operational empty state.
#[derive(IntoElement)]
pub struct EmptyState {
    title: String,
    detail: String,
    icon: Option<IconName>,
}

impl EmptyState {
    pub fn new(title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            detail: detail.into(),
            icon: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

impl RenderOnce for EmptyState {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap_1()
            .px(px(space::LG))
            .py(px(space::XL))
            .text_center()
            .when_some(self.icon, |this, icon| {
                this.child(
                    div().mb_1().child(
                        Icon::new(icon)
                            .size(IconSize::Large)
                            .color(theme.text_muted),
                    ),
                )
            })
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text_secondary)
                    .child(self.title),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_muted)
                    .child(self.detail),
            )
    }
}
