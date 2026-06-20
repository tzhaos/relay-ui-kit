use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::{contract::BORDER_WIDTH, icon::IconName, theme::ActiveTheme};

use super::shared::feedback_icon;

/// A compact inline error message for forms and terminal launch failures.
#[derive(IntoElement)]
pub struct InlineError {
    message: String,
    detail: Option<String>,
}

impl InlineError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            detail: None,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

impl RenderOnce for InlineError {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .flex()
            .items_start()
            .gap_2()
            .text_color(theme.danger)
            .child(feedback_icon(IconName::X, theme.danger))
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(BORDER_WIDTH))
                    .child(
                        div()
                            .text_sm()
                            .line_height(px(18.0))
                            .font_weight(FontWeight::MEDIUM)
                            .child(self.message),
                    )
                    .when_some(self.detail, |this, detail| {
                        this.child(
                            div()
                                .text_xs()
                                .line_height(px(16.0))
                                .text_color(theme.text_muted)
                                .child(detail),
                        )
                    }),
            )
    }
}
