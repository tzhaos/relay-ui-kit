use gpui::{App, FontWeight, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px};

use relay_ui_core::theme::{ActiveTheme, radius};

/// A tiny tooltip body for GPUI tooltip views.
#[derive(IntoElement)]
pub struct TooltipBody {
    text: String,
}

impl TooltipBody {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for TooltipBody {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .px_2()
            .py_1()
            .rounded(px(radius::MD))
            .bg(theme.text)
            .text_color(theme.app_bg)
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .child(self.text)
    }
}
