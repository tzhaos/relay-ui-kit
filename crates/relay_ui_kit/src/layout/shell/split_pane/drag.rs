use gpui::DragMoveEvent;

use super::{
    SplitAxis,
    geometry::{clamp_split_size, snap_split_size},
};

#[derive(Clone)]
pub(super) struct DraggedSplitPane {
    pub id: &'static str,
}

pub(super) fn split_size_from_drag(
    event: &DragMoveEvent<DraggedSplitPane>,
    axis: SplitAxis,
    min_first: f32,
    min_second: f32,
) -> f32 {
    let total = match axis {
        SplitAxis::Horizontal => f32::from(event.bounds.size.width),
        SplitAxis::Vertical => f32::from(event.bounds.size.height),
    };
    let raw = match axis {
        SplitAxis::Horizontal => f32::from(event.event.position.x - event.bounds.left()),
        SplitAxis::Vertical => f32::from(event.event.position.y - event.bounds.top()),
    };
    snap_split_size(clamp_split_size(raw, total, min_first, min_second))
}
