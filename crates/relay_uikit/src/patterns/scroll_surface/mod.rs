mod scrollbar;
mod state;
mod thumb;

use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, space};

/// Width of the reserved scrollbar gutter area.
const SCROLL_GUTTER_WIDTH: f32 = 10.0;

use scrollbar::scrollbar_layer;
use state::{ScrollSurfaceState, schedule_scroll_activity_check};

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
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let id = self.id.clone();
        let state = window.use_keyed_state((id.clone(), "scroll-state"), cx, |_, _| {
            ScrollSurfaceState::new()
        });
        let snapshot = state.read(cx).snapshot();

        let state_for_scroll = state.clone();
        let state_for_hover = state.clone();
        let mut scroller = div()
            .id((id.clone(), "content"))
            .size_full()
            .min_h_0()
            .track_scroll(&snapshot.handle)
            .overflow_y_scroll()
            .on_scroll_wheel(move |event, window, cx| {
                let delta_y = f32::from(event.delta.pixel_delta(window.line_height()).y);
                if delta_y == 0.0 {
                    return;
                }
                let should_check_activity = state_for_scroll
                    .update(cx, |state, _cx| state.schedule_activity_check_if_needed());
                if should_check_activity {
                    schedule_scroll_activity_check(state_for_scroll.clone(), window);
                }
            })
            .on_hover(move |hovered, _window, cx| {
                state_for_hover.update(cx, |state, cx| {
                    if state.set_hovered(*hovered) {
                        cx.notify();
                    }
                });
            })
            .when(self.reserve_gutter, |this| {
                this.scrollbar_width(px(SCROLL_GUTTER_WIDTH))
                    .pr(px(space::SM))
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
                this.child(scrollbar_layer(
                    (id.clone(), "scrollbar").into(),
                    state.clone(),
                    snapshot.thumb,
                    theme.text_muted,
                    theme.border.opacity(0.72),
                    snapshot.thumb_opacity,
                ))
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
