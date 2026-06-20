mod state;
mod thumb;

use gpui::{
    AnyElement, App, AppContext as _, ElementId, Empty, InteractiveElement, IntoElement,
    MouseButton, ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use relay_ui_core::theme::{ActiveTheme, space};

/// Width of the reserved scrollbar gutter area.
const SCROLL_GUTTER_WIDTH: f32 = 10.0;

use state::{ScrollSurfaceState, schedule_scroll_decay};
use thumb::{THUMB_WIDTH, scroll_rail};

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
                let should_schedule_decay = state_for_scroll.update(cx, |state, cx| {
                    if delta_y != 0.0 {
                        state.mark_scrolling();
                        cx.notify();
                    }
                    state.schedule_decay_if_needed()
                });

                if should_schedule_decay {
                    schedule_scroll_decay(state_for_scroll.clone(), window);
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

        let outer = div()
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
                        let state_for_thumb = state.clone();
                        let thumb_id = (id.clone(), "thumb");
                        this.child(
                            div()
                                .id(thumb_id)
                                .absolute()
                                .right(px(space::XXS))
                                .top(px(thumb.top))
                                .w(px(THUMB_WIDTH))
                                .h(px(thumb.height))
                                .rounded_full()
                                .bg(theme.text_muted.opacity(snapshot.thumb_opacity))
                                .cursor_pointer()
                                .on_drag(
                                    DraggedScrollThumb {
                                        id: id.clone(),
                                        thumb_top: thumb.top,
                                        thumb_height: thumb.height,
                                    },
                                    move |drag, cursor_offset, window, cx| {
                                        let mouse_y = f32::from(window.mouse_position().y);
                                        let click_y = f32::from(cursor_offset.y);
                                        let should_schedule_decay =
                                            state_for_thumb.update(cx, |state, cx| {
                                                state.start_thumb_drag(
                                                    mouse_y,
                                                    drag.thumb_top,
                                                    drag.thumb_height,
                                                    click_y,
                                                );
                                                state.update_thumb_drag(mouse_y);
                                                state.mark_scrolling();
                                                cx.notify();
                                                state.schedule_decay_if_needed()
                                            });
                                        if should_schedule_decay {
                                            schedule_scroll_decay(state_for_thumb.clone(), window);
                                        }
                                        cx.new(|_| Empty)
                                    },
                                ),
                        )
                    })
            });

        let state_for_drag_move = state.clone();
        let state_for_drop = state.clone();
        let drag_id = id.clone();
        let drop_id = id;

        outer
            .on_drag_move::<DraggedScrollThumb>(move |event, window, cx| {
                if event.drag(cx).id != drag_id {
                    return;
                }

                let mouse_y = f32::from(event.event.position.y);
                let should_schedule_decay = state_for_drag_move.update(cx, |state, cx| {
                    let changed = state.update_thumb_drag(mouse_y);
                    if changed {
                        state.mark_scrolling();
                        cx.notify();
                    }
                    changed && state.schedule_decay_if_needed()
                });
                if should_schedule_decay {
                    schedule_scroll_decay(state_for_drag_move.clone(), window);
                }
                cx.stop_propagation();
            })
            .on_drop::<DraggedScrollThumb>(move |drag, _window, cx| {
                if drag.id == drop_id {
                    state_for_drop.update(cx, |state, cx| {
                        state.end_thumb_drag();
                        cx.notify();
                    });
                    cx.stop_propagation();
                }
            })
            .on_mouse_up_out(MouseButton::Left, move |_event, _window, cx| {
                state.update(cx, |state, cx| {
                    state.end_thumb_drag();
                    cx.notify();
                });
            })
    }
}

#[derive(Clone)]
struct DraggedScrollThumb {
    id: ElementId,
    thumb_top: f32,
    thumb_height: f32,
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
