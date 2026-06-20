use gpui::{App, IntoElement, RenderOnce, Styled, Window, div, px};

use crate::{theme::ActiveTheme, theme::BORDER_WIDTH};

/// A 1px hairline divider.
#[derive(IntoElement)]
pub struct Divider {
    vertical: bool,
}

impl Divider {
    pub fn horizontal() -> Self {
        Self { vertical: false }
    }

    pub fn vertical() -> Self {
        Self { vertical: true }
    }
}

impl RenderOnce for Divider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = cx.theme().border;
        if self.vertical {
            div().w(px(BORDER_WIDTH)).h_full().bg(color)
        } else {
            div().h(px(BORDER_WIDTH)).w_full().bg(color)
        }
    }
}
