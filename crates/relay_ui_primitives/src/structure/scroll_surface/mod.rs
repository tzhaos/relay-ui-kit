mod state;
mod thumb;

use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, space};

/// Width of the reserved scrollbar gutter area.
const SCROLL_GUTTER_WIDTH: f32 = 10.0;

use state::{ScrollSurfaceState, schedule_scroll_decay, schedule_smooth_scroll};
use thumb::{THUMB_WIDTH, scroll_rail};

/// A stable vertical scrolling surface with Relay's standard scroll affordance.
#[derive(IntoElement)]
pub struct ScrollSurface {
    id: ElementId,
    content: AnyElement,
    reserve_gutter: bool,
    show_rail: bool,
    smooth_wheel: bool,
    max_height: Option<f32>,
}

impl ScrollSurface {
    pub fn new(id: impl Into<ElementId>, content: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            content: content.into_any_element(),
            reserve_gutter: true,
            show_rail: true,
            smooth_wheel: true,
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

    pub fn smooth_wheel(mut self, smooth: bool) -> Self {
        self.smooth_wheel = smooth;
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
        let id = self.id;
        let state = window.use_keyed_state((id.clone(), "scroll-state"), cx, |_, _| {
            ScrollSurfaceState::new()
        });
        let snapshot = state.read(cx).snapshot();

        let state_for_scroll = state.clone();
        let state_for_hover = state.clone();
        let smooth_wheel = self.smooth_wheel;
        let mut scroller = div()
            .id((id, "content"))
            .size_full()
            .min_h_0()
            .track_scroll(&snapshot.handle)
            .overflow_y_scroll()
            .on_scroll_wheel(move |event, window, cx| {
                let smooth_delta = if smooth_wheel && !event.delta.precise() {
                    Some(f32::from(event.delta.pixel_delta(window.line_height()).y))
                } else {
                    None
                };

                let (should_schedule_decay, should_schedule_smooth, consumed) = state_for_scroll
                    .update(cx, |state, cx| {
                        let smooth_started =
                            smooth_delta.is_some_and(|delta_y| state.start_smooth_scroll(delta_y));
                        let should_schedule_smooth =
                            smooth_started && state.schedule_smooth_if_needed();

                        state.mark_scrolling();
                        cx.notify();

                        (
                            state.schedule_decay_if_needed(),
                            should_schedule_smooth,
                            smooth_started,
                        )
                    });

                if should_schedule_decay {
                    schedule_scroll_decay(state_for_scroll.clone(), window);
                }

                if should_schedule_smooth {
                    schedule_smooth_scroll(state_for_scroll.clone(), window);
                }

                if consumed {
                    cx.stop_propagation();
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
                this.child(scroll_rail(theme.border.opacity(0.72)))
                    .when_some(snapshot.thumb, |this, thumb| {
                        this.child(
                            div()
                                .absolute()
                                .right(px(2.0))
                                .top(px(thumb.top))
                                .w(px(THUMB_WIDTH))
                                .h(px(thumb.height))
                                .rounded_full()
                                .bg(theme.text_muted.opacity(snapshot.thumb_opacity)),
                        )
                    })
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

    #[test]
    fn scroll_surface_smooths_wheel_by_default() {
        let surface = ScrollSurface::new("scroll", div());

        assert!(surface.smooth_wheel);
    }
}
