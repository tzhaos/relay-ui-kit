use gpui::{App, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px};

use crate::{
    theme::{ActiveTheme, radius},
    tone::Tone,
};

/// A compact numeric badge for unread counts, file counts, and tab counts.
#[derive(IntoElement)]
pub struct CountBadge {
    count: usize,
    max: usize,
    tone: Tone,
}

impl CountBadge {
    pub fn new(count: usize) -> Self {
        Self {
            count,
            max: 99,
            tone: Tone::Muted,
        }
    }

    pub fn max(mut self, max: usize) -> Self {
        self.max = max;
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn label(&self) -> String {
        if self.count > self.max {
            format!("{}+", self.max)
        } else {
            self.count.to_string()
        }
    }
}

impl RenderOnce for CountBadge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        if self.count == 0 {
            return div().into_any_element();
        }
        let theme = *cx.theme();
        let fg = self.tone.fg(&theme);
        div()
            .h(px(18.0))
            .min_w(px(18.0))
            .px(px(5.0))
            .rounded(px(radius::SM))
            .border_1()
            .border_color(theme.border)
            .bg(theme.panel_alt)
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(10.0))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(fg)
            .child(self.label())
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_badge_formats_overflow() {
        let badge = CountBadge::new(128).max(99);

        assert_eq!(badge.label(), "99+");
    }
}
