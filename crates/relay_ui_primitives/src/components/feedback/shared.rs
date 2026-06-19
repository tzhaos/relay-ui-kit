use gpui::{ParentElement, Styled, div, px};

use crate::{
    icon::{Icon, IconName, IconSize},
    tone::Tone,
};

pub(super) fn feedback_icon(icon: IconName, color: gpui::Hsla) -> gpui::Div {
    div()
        .size(px(18.0))
        .mt(px(1.0))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .child(Icon::new(icon).size(IconSize::Small).color(color))
}

pub(super) fn tone_icon(tone: Tone) -> IconName {
    match tone {
        Tone::Accent => IconName::Check,
        Tone::Warning => IconName::CircleDot,
        Tone::Danger => IconName::X,
        Tone::Info | Tone::Muted | Tone::Secondary => IconName::CircleDot,
    }
}

pub(super) fn progress_ratio(value: f32, max: f32) -> f32 {
    if max <= 0.0 {
        return 0.0;
    }

    (value / max).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_ratio_clamps_overflow() {
        assert_eq!(progress_ratio(12.0, 10.0), 1.0);
    }

    #[test]
    fn progress_ratio_handles_zero_max() {
        assert_eq!(progress_ratio(10.0, 0.0), 0.0);
    }
}
