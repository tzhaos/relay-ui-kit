use std::rc::Rc;

use gpui::{
    AnyElement, App, AppContext, DragMoveEvent, Empty, InteractiveElement, IntoElement,
    MouseButton, ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, space};

/// Split direction for [`SplitPane`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone)]
struct DraggedSplitPane {
    id: &'static str,
}

const SPLIT_DRAG_STEP: f32 = 1.0;

type ResizeHandler = Rc<dyn Fn(f32, &mut Window, &mut App) + 'static>;
type ResizeEndHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

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

/// A two-pane layout with a draggable divider.
#[derive(IntoElement)]
pub struct SplitPane {
    id: &'static str,
    axis: SplitAxis,
    first: AnyElement,
    second: AnyElement,
    first_size: f32,
    min_first: f32,
    min_second: f32,
    on_resize: Option<ResizeHandler>,
    on_resize_end: Option<ResizeEndHandler>,
}

impl SplitPane {
    pub fn new(id: &'static str, first: impl IntoElement, second: impl IntoElement) -> Self {
        Self {
            id,
            axis: SplitAxis::Horizontal,
            first: first.into_any_element(),
            second: second.into_any_element(),
            first_size: space::RAIL_WIDTH,
            min_first: 220.0,
            min_second: 420.0,
            on_resize: None,
            on_resize_end: None,
        }
    }

    pub fn axis(mut self, axis: SplitAxis) -> Self {
        self.axis = axis;
        self
    }

    pub fn first_size(mut self, first_size: f32) -> Self {
        self.first_size = first_size;
        self
    }

    pub fn min_sizes(mut self, first: f32, second: f32) -> Self {
        self.min_first = first;
        self.min_second = second;
        self
    }

    pub fn on_resize(mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_resize = Some(Rc::new(handler));
        self
    }

    pub fn on_resize_end(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_resize_end = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for SplitPane {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let handler = self.on_resize;
        let id = self.id;
        let axis = self.axis;
        let min_first = self.min_first;
        let min_second = self.min_second;
        let first_size = snap_split_size(self.first_size);
        let resize_end = self.on_resize_end;
        let handle = split_handle(id, axis, handler.clone());

        let first = match axis {
            SplitAxis::Horizontal => div()
                .w(px(first_size))
                .h_full()
                .min_h_0()
                .flex_shrink_0()
                .child(self.first),
            SplitAxis::Vertical => div()
                .h(px(first_size))
                .w_full()
                .min_w_0()
                .flex_shrink_0()
                .child(self.first),
        };
        let second = div().flex_1().min_w_0().min_h_0().child(self.second);

        div()
            .id(id)
            .size_full()
            .min_w_0()
            .min_h_0()
            .flex()
            .when(axis == SplitAxis::Vertical, |this| this.flex_col())
            .bg(theme.app_bg)
            .child(first)
            .child(handle)
            .child(second)
            .when_some(handler, |this, handler| {
                this.on_drag_move::<DraggedSplitPane>(move |event, window, cx| {
                    if event.drag(cx).id != id {
                        return;
                    }
                    let next = split_size_from_drag(event, axis, min_first, min_second);
                    if should_emit_resize(first_size, next) {
                        handler(next, window, cx);
                    }
                    cx.stop_propagation();
                })
            })
            .when_some(resize_end, |this, handler| {
                this.on_drop::<DraggedSplitPane>(move |drag, window, cx| {
                    if drag.id == id {
                        handler(window, cx);
                        cx.stop_propagation();
                    }
                })
            })
    }
}

fn split_size_from_drag(
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

fn split_handle(id: &'static str, axis: SplitAxis, handler: Option<ResizeHandler>) -> gpui::Div {
    let divider = match axis {
        SplitAxis::Horizontal => div().w(px(1.0)).h_full(),
        SplitAxis::Vertical => div().h(px(1.0)).w_full(),
    };
    let interactive = match axis {
        SplitAxis::Horizontal => div()
            .id(gpui::SharedString::from(format!("split-handle-{id}")))
            .absolute()
            .left(px(-3.0))
            .top_0()
            .w(px(7.0))
            .h_full()
            .cursor_col_resize(),
        SplitAxis::Vertical => div()
            .id(gpui::SharedString::from(format!("split-handle-{id}")))
            .absolute()
            .top(px(-3.0))
            .left_0()
            .h(px(7.0))
            .w_full()
            .cursor_row_resize(),
    };

    divider
        .relative()
        .flex_shrink_0()
        .bg(gpui::transparent_black())
        .child(
            interactive
                .when_some(handler, |this, _handler| {
                    this.on_mouse_down(MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .on_drag(DraggedSplitPane { id }, |_, _, _window, cx| {
                        cx.new(|_| Empty)
                    })
                    .on_drop::<DraggedSplitPane>(|_, _, cx| {
                        cx.stop_propagation();
                    })
                })
                .hover(move |style| style.bg(gpui::black().opacity(0.08))),
        )
}

fn clamp_split_size(raw: f32, total: f32, min_first: f32, min_second: f32) -> f32 {
    if total <= min_first + min_second {
        return min_first.min(total.max(0.0));
    }
    raw.clamp(min_first, total - min_second)
}

fn snap_split_size(value: f32) -> f32 {
    (value / SPLIT_DRAG_STEP).round() * SPLIT_DRAG_STEP
}

fn should_emit_resize(previous: f32, next: f32) -> bool {
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
