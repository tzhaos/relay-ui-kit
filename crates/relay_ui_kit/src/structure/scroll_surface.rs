use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, space};

/// A stable vertical scrolling surface with Relay's standard scroll affordance.
#[derive(IntoElement)]
pub struct ScrollSurface {
    id: ElementId,
    content: AnyElement,
    reserve_gutter: bool,
    show_rail: bool,
    max_height: Option<f32>,
}

impl ScrollSurface {
    pub fn new(id: impl Into<ElementId>, content: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            content: content.into_any_element(),
            reserve_gutter: true,
            show_rail: true,
            max_height: None,
        }
    }

    pub fn reserve_gutter(mut self, reserve: bool) -> Self {
        self.reserve_gutter = reserve;
        self
    }

    pub fn show_rail(mut self, show: bool) -> Self {
        self.show_rail = show;
        self
    }

    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = Some(max_height);
        self
    }
}

impl RenderOnce for ScrollSurface {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let mut scroller = div()
            .id(self.id)
            .size_full()
            .min_h_0()
            .overflow_y_scroll()
            .when(self.reserve_gutter, |this| {
                this.scrollbar_width(px(10.0)).pr(px(space::SM))
            })
            .child(self.content);
        scroller.style().restrict_scroll_to_axis = Some(true);

        div()
            .relative()
            .size_full()
            .min_h_0()
            .flex_1()
            .overflow_hidden()
            .when_some(self.max_height, |this, max_height| {
                this.max_h(px(max_height))
            })
            .child(scroller)
            .when(self.show_rail, |this| {
                this.child(
                    div()
                        .absolute()
                        .top_0()
                        .right_0()
                        .bottom_0()
                        .w(px(1.0))
                        .bg(theme.border.opacity(0.72)),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_surface_reserves_gutter_by_default() {
        let surface = ScrollSurface::new("scroll", div());

        assert!(surface.reserve_gutter);
    }

    #[test]
    fn scroll_surface_can_hide_rail() {
        let surface = ScrollSurface::new("scroll", div()).show_rail(false);

        assert!(!surface.show_rail);
    }
}
