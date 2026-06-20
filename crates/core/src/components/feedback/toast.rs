use gpui::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled,
    Window, div, prelude::FluentBuilder, px,
};

use crate::{
    motion::{MotionDirection, MotionExt},
    theme::{ActiveTheme, BORDER_WIDTH, radius},
    tone::Tone,
};

use super::shared::{feedback_icon, tone_icon};

/// A compact floating notification body.
#[derive(IntoElement)]
pub struct Toast {
    id: ElementId,
    title: String,
    detail: Option<String>,
    tone: Tone,
    animated: bool,
}

impl Toast {
    pub fn new(id: impl Into<ElementId>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            detail: None,
            tone: Tone::Info,
            animated: true,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }
}

impl RenderOnce for Toast {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let fg = self.tone.fg(&theme);
        let base = div()
            .id(self.id)
            .w(px(320.0))
            .px_3()
            .py_2()
            .flex()
            .items_start()
            .gap_2()
            .rounded(px(radius::LG))
            .border_1()
            .border_color(theme.border_strong)
            .bg(theme.panel)
            .shadow_lg()
            .child(feedback_icon(tone_icon(self.tone), fg))
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(BORDER_WIDTH))
                    .child(
                        div()
                            .truncate()
                            .text_sm()
                            .line_height(px(18.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text)
                            .child(self.title),
                    )
                    .when_some(self.detail, |this, detail| {
                        this.child(
                            div()
                                .truncate()
                                .text_xs()
                                .line_height(px(16.0))
                                .text_color(theme.text_muted)
                                .child(detail),
                        )
                    }),
            );

        if self.animated {
            base.motion_slide_in(MotionDirection::FromBottom, true)
                .into_any_element()
        } else {
            base.into_any_element()
        }
    }
}
