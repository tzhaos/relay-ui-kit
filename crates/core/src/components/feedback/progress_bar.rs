use gpui::{
    App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce, Role,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px, relative,
};

use crate::{theme::ActiveTheme, tone::Tone};

use super::shared::progress_ratio;

/// A determinate horizontal progress bar.
#[derive(IntoElement)]
pub struct ProgressBar {
    id: ElementId,
    value: f32,
    max: f32,
    tone: Tone,
    label: Option<String>,
}

impl ProgressBar {
    pub fn new(id: impl Into<ElementId>, value: f32, max: f32) -> Self {
        Self {
            id: id.into(),
            value,
            max,
            tone: Tone::Accent,
            label: None,
        }
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn ratio(&self) -> f32 {
        progress_ratio(self.value, self.max)
    }
}

impl RenderOnce for ProgressBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let ratio = self.ratio();
        let fg = self.tone.fg(&theme);
        let bar = div()
            .id(self.id)
            .h(px(6.0))
            .w_full()
            .rounded_full()
            .bg(theme.panel_alt)
            .role(Role::ProgressIndicator)
            .border_1()
            .border_color(theme.border)
            .child(div().h_full().rounded_full().bg(fg).w(relative(ratio)));

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_1()
            .when_some(self.label, |this, label| {
                this.child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text_secondary)
                        .child(label),
                )
            })
            .child(bar)
    }
}
