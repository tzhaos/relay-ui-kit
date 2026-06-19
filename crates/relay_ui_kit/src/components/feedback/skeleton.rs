use gpui::{App, ElementId, InteractiveElement, IntoElement, RenderOnce, Styled, Window, div, px};

use crate::{
    motion::MotionExt,
    theme::{ActiveTheme, radius},
};

/// A subtle placeholder block for loading rows and panes.
#[derive(IntoElement)]
pub struct Skeleton {
    id: ElementId,
    width: f32,
    height: f32,
    animated: bool,
}

impl Skeleton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            width: 160.0,
            height: 12.0,
            animated: true,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }
}

impl RenderOnce for Skeleton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let base = div()
            .id(self.id)
            .w(px(self.width))
            .h(px(self.height))
            .rounded(px(radius::SM))
            .bg(theme.panel_alt)
            .border_1()
            .border_color(theme.border);

        if self.animated {
            base.motion_pulse(0.45, 0.86).into_any_element()
        } else {
            base.into_any_element()
        }
    }
}
