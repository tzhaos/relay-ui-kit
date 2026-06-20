use super::geometry::{should_emit_resize, snap_split_size};

/// Host-owned split sizing state with pixel-stable resize previews.
#[derive(Debug, Clone, Copy)]
pub struct SplitPaneState {
    committed_first_size: f32,
    visible_first_size: f32,
    resizing: bool,
}

impl SplitPaneState {
    pub fn new(first_size: f32) -> Self {
        let first_size = snap_split_size(first_size);
        Self {
            committed_first_size: first_size,
            visible_first_size: first_size,
            resizing: false,
        }
    }

    pub fn first_size(&self) -> f32 {
        self.visible_first_size
    }

    pub fn committed_first_size(&self) -> f32 {
        self.committed_first_size
    }

    pub fn is_resizing(&self) -> bool {
        self.resizing
    }

    pub fn set_first_size(&mut self, first_size: f32) {
        let first_size = snap_split_size(first_size);
        self.committed_first_size = first_size;
        self.visible_first_size = first_size;
        self.resizing = false;
    }

    pub fn resize_to(&mut self, first_size: f32) -> bool {
        self.preview_resize_to(first_size)
    }

    pub fn preview_resize_to(&mut self, first_size: f32) -> bool {
        let next = snap_split_size(first_size);
        if should_emit_resize(self.visible_first_size, next) {
            self.visible_first_size = next;
            self.resizing = true;
            true
        } else {
            false
        }
    }

    pub fn commit_resize(&mut self) -> bool {
        let changed = should_emit_resize(self.committed_first_size, self.visible_first_size);
        self.committed_first_size = self.visible_first_size;
        self.resizing = false;
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_state_reports_when_size_changes() {
        let mut state = SplitPaneState::new(300.2);

        assert!(state.preview_resize_to(300.6));
    }

    #[test]
    fn split_state_skips_subpixel_resize() {
        let mut state = SplitPaneState::new(300.2);

        assert!(!state.preview_resize_to(300.4));
    }

    #[test]
    fn split_state_keeps_committed_size_until_drop() {
        let mut state = SplitPaneState::new(300.0);

        assert!(state.preview_resize_to(340.0));
        assert_eq!(state.first_size(), 340.0);
        assert_eq!(state.committed_first_size(), 300.0);
        assert!(state.is_resizing());
    }

    #[test]
    fn split_state_commits_visible_size() {
        let mut state = SplitPaneState::new(300.0);

        state.preview_resize_to(340.0);

        assert!(state.commit_resize());
        assert_eq!(state.committed_first_size(), 340.0);
        assert!(!state.is_resizing());
    }
}
