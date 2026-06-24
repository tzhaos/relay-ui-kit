use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    theme::{ActiveTheme, radius, space},
    tone::Tone,
};

/// Fill treatment for a [`Badge`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeStyle {
    /// Soft tinted fill in the tone's color family.
    Soft,
    /// Quiet neutral fill with a 1px border.
    Outline,
}

/// A small label chip for metadata and status.
#[derive(IntoElement)]
pub struct Badge {
    label: String,
    tone: Tone,
    style: BadgeStyle,
    icon: Option<IconName>,
}

impl Badge {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            tone: Tone::Muted,
            style: BadgeStyle::Outline,
            icon: None,
        }
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn style(mut self, style: BadgeStyle) -> Self {
        self.style = style;
        self
    }

    pub fn soft(self) -> Self {
        self.style(BadgeStyle::Soft)
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let fg = self.tone.fg(&theme);
        let (bg, border) = match self.style {
            BadgeStyle::Soft => (self.tone.soft_bg(&theme), self.tone.soft_border(&theme)),
            BadgeStyle::Outline => (theme.panel_alt, theme.border),
        };

        div()
            .h(px(20.0))
            .max_w(px(240.0))
            .px(px(space::SM))
            .rounded(px(radius::SM))
            .border_1()
            .border_color(border)
            .bg(bg)
            .overflow_hidden()
            .flex()
            .items_center()
            .gap_1()
            .text_color(fg)
            .text_size(px(11.0))
            .font_weight(FontWeight::MEDIUM)
            .when_some(self.icon, |this, icon| {
                this.child(Icon::new(icon).size(IconSize::XSmall).color(fg))
            })
            .child(div().truncate().child(self.label))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_style_builder_stores_explicit_style() {
        let badge = Badge::new("Relay").style(BadgeStyle::Soft);

        assert_eq!(badge.style, BadgeStyle::Soft);
    }
}
