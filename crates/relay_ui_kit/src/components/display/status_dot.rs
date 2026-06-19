use gpui::{App, IntoElement, RenderOnce, Styled, Window, div, px};

use crate::{theme::ActiveTheme, tone::Tone};

/// A small circular status indicator.
#[derive(IntoElement)]
pub struct StatusDot {
    tone: Tone,
    size: f32,
}

impl StatusDot {
    pub fn new(tone: Tone) -> Self {
        Self { tone, size: 7.0 }
    }

    pub fn size(mut self, px: f32) -> Self {
        self.size = px;
        self
    }
}

impl RenderOnce for StatusDot {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = self.tone.fg(cx.theme());
        div().size(px(self.size)).rounded_full().bg(color)
    }
}
