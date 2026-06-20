use gpui::{Entity, ScrollHandle, Window, point, px};

use super::thumb::{ScrollThumbMetrics, thumb_metrics_from_values};

const SCROLL_ACTIVITY_FRAMES: u8 = 14;

pub(super) struct ScrollSurfaceState {
    handle: ScrollHandle,
    hovered: bool,
    activity_frames: u8,
    decay_scheduled: bool,
    thumb_drag: Option<ThumbDragState>,
}

impl ScrollSurfaceState {
    pub(super) fn new() -> Self {
        Self {
            handle: ScrollHandle::new(),
            hovered: false,
            activity_frames: 0,
            decay_scheduled: false,
            thumb_drag: None,
        }
    }

    pub(super) fn snapshot(&self) -> ScrollSurfaceSnapshot {
        let thumb = thumb_metrics_from_values(
            f32::from(self.handle.bounds().size.height),
            f32::from(self.handle.max_offset().y),
            f32::from(self.handle.offset().y),
        );
        ScrollSurfaceSnapshot {
            handle: self.handle.clone(),
            thumb,
            thumb_opacity: self.thumb_opacity(),
        }
    }

    pub(super) fn set_hovered(&mut self, hovered: bool) -> bool {
        let changed = self.hovered != hovered;
        self.hovered = hovered;
        changed
    }

    pub(super) fn mark_scrolling(&mut self) -> bool {
        let was_visible = self.activity_frames > 0;
        self.activity_frames = SCROLL_ACTIVITY_FRAMES;
        !was_visible
    }

    pub(super) fn schedule_decay_if_needed(&mut self) -> bool {
        if self.activity_frames > 0 && !self.decay_scheduled {
            self.decay_scheduled = true;
            true
        } else {
            false
        }
    }

    pub(super) fn start_thumb_drag(
        &mut self,
        mouse_window_y: f32,
        thumb_top: f32,
        thumb_height: f32,
        mouse_offset_in_thumb: f32,
    ) {
        let viewport_height = f32::from(self.handle.bounds().size.height);
        if viewport_height <= 0.0 || thumb_height <= 0.0 {
            return;
        }
        self.thumb_drag = Some(ThumbDragState {
            rail_top: mouse_window_y - thumb_top - mouse_offset_in_thumb,
            mouse_offset: mouse_offset_in_thumb,
            viewport_height,
            thumb_height,
        });
    }

    pub(super) fn update_thumb_drag(&mut self, mouse_window_y: f32) -> bool {
        let Some(drag) = self.thumb_drag else {
            return false;
        };
        let max_scroll = f32::from(self.handle.max_offset().y);
        let next_y = drag_scroll_offset(mouse_window_y, drag, max_scroll);
        let current_y = f32::from(self.handle.offset().y);
        if (next_y - current_y).abs() <= f32::EPSILON {
            return false;
        }
        self.set_scroll_y(next_y);
        true
    }

    pub(super) fn end_thumb_drag(&mut self) {
        self.thumb_drag = None;
    }

    // ------------------------------------------------------------------
    // Scrollbar visibility decay
    // ------------------------------------------------------------------

    fn tick_activity(&mut self) -> bool {
        self.decay_scheduled = false;
        if self.activity_frames == 0 {
            return false;
        }
        self.activity_frames -= 1;
        if self.activity_frames > 0 {
            self.decay_scheduled = true;
        }
        true
    }

    fn should_continue_decay(&self) -> bool {
        self.activity_frames > 0
    }

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    fn set_scroll_y(&mut self, y: f32) {
        let offset = self.handle.offset();
        self.handle.set_offset(point(offset.x, px(y)));
    }

    fn thumb_opacity(&self) -> f32 {
        if self.hovered {
            return 0.72;
        }
        let activity = f32::from(self.activity_frames) / f32::from(SCROLL_ACTIVITY_FRAMES);
        0.26 + activity * 0.46
    }
}

pub(super) struct ScrollSurfaceSnapshot {
    pub(super) handle: ScrollHandle,
    pub(super) thumb: Option<ScrollThumbMetrics>,
    pub(super) thumb_opacity: f32,
}

pub(super) fn schedule_scroll_decay(state: Entity<ScrollSurfaceState>, window: &mut Window) {
    window.on_next_frame(move |window, cx| {
        let should_continue = state.update(cx, |state, cx| {
            if state.tick_activity() {
                cx.notify();
            }
            state.should_continue_decay()
        });
        if should_continue {
            schedule_scroll_decay(state, window);
        }
    });
}

#[derive(Clone, Copy)]
struct ThumbDragState {
    rail_top: f32,
    mouse_offset: f32,
    viewport_height: f32,
    thumb_height: f32,
}

fn drag_scroll_offset(mouse_window_y: f32, drag: ThumbDragState, max_scroll_y: f32) -> f32 {
    if max_scroll_y <= 0.0 {
        return 0.0;
    }

    let rail_y = mouse_window_y - drag.rail_top;
    let thumb_range = (drag.viewport_height - drag.thumb_height).max(0.0);
    let thumb_top = (rail_y - drag.mouse_offset).clamp(0.0, thumb_range);
    let progress = if thumb_range > 0.0 {
        thumb_top / thumb_range
    } else {
        0.0
    };

    -progress * max_scroll_y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_activity_schedules_decay_once() {
        let mut state = ScrollSurfaceState::new();
        assert!(state.mark_scrolling());
        assert!(state.schedule_decay_if_needed());
        assert!(!state.schedule_decay_if_needed());
    }

    #[test]
    fn thumb_drag_maps_pointer_position_to_negative_scroll_offset() {
        let drag = ThumbDragState {
            rail_top: 20.0,
            mouse_offset: 10.0,
            viewport_height: 200.0,
            thumb_height: 50.0,
        };

        assert_eq!(drag_scroll_offset(105.0, drag, 300.0), -150.0);
    }
}
