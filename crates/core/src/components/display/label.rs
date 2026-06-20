use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::{theme::ActiveTheme, tone::Tone};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelSize {
    XSmall,
    Small,
    Medium,
}

impl LabelSize {
    fn px(self) -> f32 {
        match self {
            LabelSize::XSmall => 11.0,
            LabelSize::Small => 12.0,
            LabelSize::Medium => 13.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelColor {
    Primary,
    Secondary,
    Muted,
    Tone(Tone),
}

/// A reusable text label for dense workbench chrome.
#[derive(IntoElement)]
pub struct Label {
    text: String,
    size: LabelSize,
    color: LabelColor,
    strong: bool,
    truncate: bool,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            size: LabelSize::Small,
            color: LabelColor::Primary,
            strong: false,
            truncate: false,
        }
    }

    pub fn size(mut self, size: LabelSize) -> Self {
        self.size = size;
        self
    }

    pub fn muted(mut self) -> Self {
        self.color = LabelColor::Muted;
        self
    }

    pub fn secondary(mut self) -> Self {
        self.color = LabelColor::Secondary;
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.color = LabelColor::Tone(tone);
        self
    }

    pub fn strong(mut self) -> Self {
        self.strong = true;
        self
    }

    pub fn truncate(mut self) -> Self {
        self.truncate = true;
        self
    }
}

impl RenderOnce for Label {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let color = match self.color {
            LabelColor::Primary => theme.text,
            LabelColor::Secondary => theme.text_secondary,
            LabelColor::Muted => theme.text_muted,
            LabelColor::Tone(tone) => tone.fg(&theme),
        };

        div()
            .min_w_0()
            .text_size(px(self.size.px()))
            .text_color(color)
            .font_weight(if self.strong {
                FontWeight::MEDIUM
            } else {
                FontWeight::NORMAL
            })
            .when(self.truncate, |this| this.truncate())
            .child(self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_defaults_to_small_primary_text() {
        let label = Label::new("Relay");

        assert_eq!(label.size, LabelSize::Small);
        assert_eq!(label.color, LabelColor::Primary);
    }
}
