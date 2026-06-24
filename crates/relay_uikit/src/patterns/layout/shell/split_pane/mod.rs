//! Resizable two-pane shell layout for Relay desktop surfaces.

use gpui::{
    AnyElement, App, ElementId, Entity, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    Styled, Window, div, prelude::FluentBuilder, px,
};

mod drag;
mod geometry;
mod handle;
mod handlers;
mod state;

use crate::{
    interaction::{SharedChangeHandler, SharedDismissHandler},
    theme::{ActiveTheme, space},
};

use drag::{DraggedSplitPane, split_size_from_drag};
use geometry::{should_emit_resize, snap_split_size};
use handle::{SplitHandleContext, render_split_handle};
use handlers::{resize_end_handler, resize_handler};
pub use state::SplitPaneState;

/// Split direction for [`SplitPane`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitAxis {
    /// Lay out panes left-to-right with a vertical divider.
    Horizontal,
    /// Lay out panes top-to-bottom with a horizontal divider.
    Vertical,
}

/// A two-pane layout with a draggable divider.
#[derive(IntoElement)]
pub struct SplitPane {
    id: ElementId,
    axis: SplitAxis,
    first: AnyElement,
    second: AnyElement,
    first_size: f32,
    state: Option<Entity<SplitPaneState>>,
    min_first: f32,
    min_second: f32,
    on_resize: Option<SharedChangeHandler<f32>>,
    on_resize_end: Option<SharedDismissHandler>,
}

impl SplitPane {
    /// Create a split pane with two children and a stable root [`ElementId`].
    pub fn new(
        id: impl Into<ElementId>,
        first: impl IntoElement,
        second: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            axis: SplitAxis::Horizontal,
            first: first.into_any_element(),
            second: second.into_any_element(),
            first_size: space::RAIL_WIDTH,
            state: None,
            min_first: 220.0,
            min_second: 420.0,
            on_resize: None,
            on_resize_end: None,
        }
    }

    /// Choose the axis used to place the two panes.
    pub fn axis(mut self, axis: SplitAxis) -> Self {
        self.axis = axis;
        self
    }

    /// Set the initial size for the first pane before any drag state is restored.
    pub fn first_size(mut self, first_size: f32) -> Self {
        self.first_size = first_size;
        self
    }

    /// Persist resize state in a host-owned [`SplitPaneState`] entity.
    pub fn state(mut self, state: Entity<SplitPaneState>) -> Self {
        self.state = Some(state);
        self
    }

    /// Set minimum sizes for each pane.
    ///
    /// # Panics
    ///
    /// In debug builds, panics if `first + second` exceeds a reasonable total
    /// (1600 px), since the two panes would never both satisfy their minimums.
    pub fn min_sizes(mut self, first: f32, second: f32) -> Self {
        debug_assert!(
            first + second <= 1600.0,
            "SplitPane min_sizes ({first} + {second} = {}) exceeds reasonable total; \
             panes cannot both satisfy minimums",
            first + second
        );
        self.min_first = first;
        self.min_second = second;
        self
    }

    /// Observe live resize previews as the divider moves.
    pub fn on_resize(mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_resize = Some(std::rc::Rc::new(handler));
        self
    }

    /// Observe the end of a drag or keyboard resize interaction.
    pub fn on_resize_end(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_resize_end = Some(std::rc::Rc::new(handler));
        self
    }
}

impl RenderOnce for SplitPane {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let id = self.id;
        let root_id = id.clone();
        let axis = self.axis;
        let min_first = self.min_first;
        let min_second = self.min_second;
        let state = self.state;
        let state_first_size = state
            .as_ref()
            .map(|state| state.read(cx).first_size())
            .unwrap_or(self.first_size);
        let first_size = snap_split_size(state_first_size);
        let handler = resize_handler(state.clone(), self.on_resize);
        let resize_end = resize_end_handler(state, self.on_resize_end);
        let keyboard_ctx = handler.as_ref().map(|_| SplitHandleContext {
            resize: handler.clone(),
            resize_end: resize_end.clone(),
            first_size,
            min_first,
            min_second,
        });
        let handle = render_split_handle(
            id.clone(),
            axis,
            handler.is_some(),
            keyboard_ctx,
            window,
            cx,
        );

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
            .id(root_id)
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
                let drag_id = id.clone();
                this.on_drag_move::<DraggedSplitPane>(move |event, window, cx| {
                    if event.drag(cx).id != drag_id {
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
                let drag_id = id.clone();
                this.on_drop::<DraggedSplitPane>(move |drag, window, cx| {
                    if drag.id == drag_id {
                        handler(window, cx);
                        cx.stop_propagation();
                    }
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use gpui::{ElementId, div};

    use super::*;

    #[test]
    fn split_pane_accepts_owned_element_ids() {
        let pane = SplitPane::new(format!("split-{}", "review"), div(), div());

        assert_eq!(pane.id, ElementId::Name("split-review".into()));
    }
}
