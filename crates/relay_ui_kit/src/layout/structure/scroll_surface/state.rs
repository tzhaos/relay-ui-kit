use gpui::{Entity, ScrollHandle, Window, point, px};

use super::thumb::{ScrollThumbMetrics, thumb_metrics_from_values};

const SCROLL_ACTIVITY_FRAMES: u8 = 14;
const SMOOTH_SCROLL_FACTOR: f32 = 0.34;
const SMOOTH_SCROLL_EPSILON: f32 = 0.7;

pub(super) struct ScrollSurfaceState {
    handle: ScrollHandle,
    hovered: bool,
    activity_frames: u8,
    decay_scheduled: bool,
    smooth_target_y: Option<f32>,
    smooth_scheduled: bool,
}

impl ScrollSurfaceState {
    pub(super) fn new() -> Self {
        Self {
            handle: ScrollHandle::new(),
            hovered: false,
            activity_frames: 0,
            decay_scheduled: false,
            smooth_target_y: None,
            smooth_scheduled: false,
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

    pub(super) fn start_smooth_scroll(&mut self, delta_y: f32) -> bool {
        let max_scroll_y = f32::from(self.handle.max_offset().y);
        if max_scroll_y <= 0.0 || delta_y == 0.0 {
            return false;
        }

        let current_y = f32::from(self.handle.offset().y);
        let base_y = self.smooth_target_y.unwrap_or(current_y);
        let target_y = clamp_scroll_y(base_y + delta_y, max_scroll_y);

        if self.smooth_target_y.is_none() && (target_y - current_y).abs() <= SMOOTH_SCROLL_EPSILON {
            return false;
        }

        if (target_y - current_y).abs() <= SMOOTH_SCROLL_EPSILON {
            self.set_scroll_y(target_y);
            self.smooth_target_y = None;
            return true;
        }

        self.smooth_target_y = Some(target_y);
        true
    }

    pub(super) fn schedule_smooth_if_needed(&mut self) -> bool {
        if self.smooth_target_y.is_some() && !self.smooth_scheduled {
            self.smooth_scheduled = true;
            true
        } else {
            false
        }
    }

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

    fn tick_smooth_scroll(&mut self) -> bool {
        self.smooth_scheduled = false;
        let Some(target_y) = self.smooth_target_y else {
            return false;
        };

        let current_y = f32::from(self.handle.offset().y);
        let (next_y, done) = smooth_scroll_step(current_y, target_y);
        self.set_scroll_y(next_y);

        if done {
            self.smooth_target_y = None;
        } else {
            self.smooth_scheduled = true;
        }

        true
    }

    fn should_continue_smooth_scroll(&self) -> bool {
        self.smooth_target_y.is_some()
    }

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

pub(super) fn schedule_smooth_scroll(state: Entity<ScrollSurfaceState>, window: &mut Window) {
    window.on_next_frame(move |window, cx| {
        let should_continue = state.update(cx, |state, cx| {
            if state.tick_smooth_scroll() {
                cx.notify();
            }
            state.should_continue_smooth_scroll()
        });

        if should_continue {
            schedule_smooth_scroll(state, window);
        }
    });
}

fn clamp_scroll_y(value: f32, max_scroll_y: f32) -> f32 {
    value.clamp(-max_scroll_y, 0.0)
}

fn smooth_scroll_step(current_y: f32, target_y: f32) -> (f32, bool) {
    let distance = target_y - current_y;
    if distance.abs() <= SMOOTH_SCROLL_EPSILON {
        return (target_y, true);
    }

    (current_y + distance * SMOOTH_SCROLL_FACTOR, false)
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
    fn clamp_scroll_y_keeps_offset_in_scroll_bounds() {
        assert_eq!(clamp_scroll_y(-480.0, 320.0), -320.0);
    }

    #[test]
    fn smooth_scroll_step_moves_toward_target() {
        let (next, done) = smooth_scroll_step(0.0, -100.0);

        assert_eq!(next, -34.0);
        assert!(!done);
    }
}
