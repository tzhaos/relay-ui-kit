use gpui::{
    AnyElement, App, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::{
    theme::{ActiveTheme, BORDER_WIDTH, radius},
    tone::Tone,
};

use super::shared::{feedback_icon, tone_icon};

/// A compact in-flow notice for settings, empty panels, and launch diagnostics.
#[derive(IntoElement)]
pub struct Callout {
    title: String,
    detail: Option<String>,
    tone: Tone,
    action: Option<AnyElement>,
}

impl Callout {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            detail: None,
            tone: Tone::Info,
            action: None,
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

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }
}

impl RenderOnce for Callout {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let fg = self.tone.fg(&theme);

        div()
            .min_h(px(34.0))
            .px_2()
            .py_2()
            .rounded(px(radius::MD))
            .border_1()
            .border_color(self.tone.soft_border(&theme))
            .border_l_2()
            .bg(theme.panel)
            .flex()
            .items_start()
            .gap_2()
            .max_w_full()
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
                            .min_w_0()
                            .w_full()
                            .text_sm()
                            .line_height(px(18.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text)
                            .child(self.title),
                    )
                    .when_some(self.detail, |this, detail| {
                        this.child(
                            div()
                                .min_w_0()
                                .w_full()
                                .text_xs()
                                .line_height(px(16.0))
                                .text_color(theme.text_muted)
                                .child(detail),
                        )
                    }),
            )
            .when_some(self.action, |this, action| {
                this.child(div().flex_shrink_0().child(action))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callout_defaults_to_info_tone() {
        let callout = Callout::new("Notice");

        assert_eq!(callout.tone, Tone::Info);
    }
}
