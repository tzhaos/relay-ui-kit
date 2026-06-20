use gpui::{Hsla, IntoElement, Styled, div, px};

use relay_foundation::theme::BORDER_WIDTH;

/// Scroll thumb width in pixels.
pub(super) const THUMB_WIDTH: f32 = 5.0;

/// Minimum scroll thumb height to prevent it from becoming too small to interact with.
const MIN_THUMB_HEIGHT: f32 = 24.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ScrollThumbMetrics {
    pub(super) top: f32,
    pub(super) height: f32,
}

pub(super) fn scroll_rail(color: Hsla) -> impl IntoElement {
    div()
        .absolute()
        .top_0()
        .right_0()
        .bottom_0()
        .w(px(BORDER_WIDTH))
        .bg(color)
}

pub(super) fn thumb_metrics_from_values(
    viewport_height: f32,
    max_scroll_offset: f32,
    scroll_offset: f32,
) -> Option<ScrollThumbMetrics> {
    if viewport_height <= 0.0 || max_scroll_offset <= 0.0 {
        return None;
    }

    let content_height = viewport_height + max_scroll_offset;
    let thumb_height = (viewport_height * (viewport_height / content_height))
        .max(MIN_THUMB_HEIGHT)
        .min(viewport_height);
    if thumb_height >= viewport_height {
        return None;
    }

    let scroll_progress = scroll_offset.abs().min(max_scroll_offset) / max_scroll_offset;
    let top = (viewport_height - thumb_height) * scroll_progress;

    Some(ScrollThumbMetrics {
        top,
        height: thumb_height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumb_metrics_hide_without_overflow() {
        assert_eq!(thumb_metrics_from_values(320.0, 0.0, 0.0), None);
    }

    #[test]
    fn thumb_metrics_track_scroll_progress() {
        assert_eq!(
            thumb_metrics_from_values(300.0, 300.0, -150.0),
            Some(ScrollThumbMetrics {
                top: 75.0,
                height: 150.0
            })
        );
    }
}
