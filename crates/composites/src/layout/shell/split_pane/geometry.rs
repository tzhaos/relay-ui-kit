const SPLIT_DRAG_STEP: f32 = 1.0;

pub(super) fn clamp_split_size(raw: f32, total: f32, min_first: f32, min_second: f32) -> f32 {
    if total <= min_first + min_second {
        return min_first.min(total.max(0.0));
    }
    raw.clamp(min_first, total - min_second)
}

pub(super) fn snap_split_size(value: f32) -> f32 {
    (value / SPLIT_DRAG_STEP).round() * SPLIT_DRAG_STEP
}

pub(super) fn should_emit_resize(previous: f32, next: f32) -> bool {
    snap_split_size(previous) != snap_split_size(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_size_clamps_to_minimums() {
        assert_eq!(clamp_split_size(10.0, 1000.0, 220.0, 420.0), 220.0);
    }

    #[test]
    fn split_size_clamps_to_secondary_minimum() {
        assert_eq!(clamp_split_size(900.0, 1000.0, 220.0, 420.0), 580.0);
    }

    #[test]
    fn split_size_snaps_to_whole_pixels() {
        assert_eq!(snap_split_size(301.2), 301.0);
    }

    #[test]
    fn resize_event_skips_duplicate_snapped_size() {
        assert!(!should_emit_resize(300.0, 300.4));
    }

    #[test]
    fn resize_event_emits_next_snapped_size() {
        assert!(should_emit_resize(300.0, 300.6));
    }
}
