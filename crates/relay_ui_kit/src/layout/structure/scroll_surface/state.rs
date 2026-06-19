use gpui::{Entity, ScrollHandle, Window};

use super::thumb::{ScrollThumbMetrics, thumb_metrics_from_values};

const SCROLL_ACTIVITY_FRAMES: u8 = 14;

pub(super) struct ScrollSurfaceState {
    handle: ScrollHandle,
    hovered: bool,
    activity_frames: u8,
    decay_scheduled: bool,
}

impl ScrollSurfaceState {
    pub(super) fn new() -> Self {
        Self {
            handle: ScrollHandle::new(),
            hovered: false,
            activity_frames: 0,
            decay_scheduled: false,
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
}
