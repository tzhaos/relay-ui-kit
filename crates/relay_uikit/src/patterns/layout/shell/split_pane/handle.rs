use gpui::{
    App, AppContext as _, ElementId, Empty, InteractiveElement, KeyDownEvent, MouseButton,
    ParentElement, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    interaction::{SharedChangeHandler, SharedDismissHandler},
    theme::ActiveTheme,
    theme::BORDER_WIDTH,
};

use super::{SplitAxis, drag::DraggedSplitPane, geometry::clamp_split_size};

/// Keyboard step size for SplitPane resize (pixels per arrow-key press).
const KEYBOARD_RESIZE_STEP: f32 = 10.0;
/// Large step for Home/End keys (pixels).
const KEYBOARD_RESIZE_LARGE_STEP: f32 = 100.0;

pub(super) struct SplitHandleContext {
    pub resize: Option<SharedChangeHandler<f32>>,
    pub resize_end: Option<SharedDismissHandler>,
    pub first_size: f32,
    pub min_first: f32,
    pub min_second: f32,
}

pub(super) fn render_split_handle(
    id: ElementId,
    axis: SplitAxis,
    enabled: bool,
    keyboard: Option<SplitHandleContext>,
    window: &mut Window,
    cx: &mut App,
) -> gpui::Div {
    let handle_scope_id = (id.clone(), "scope");
    window.with_id(handle_scope_id, |window| {
        let is_highlighted = window.use_state(cx, |_window, _cx| false);
        let highlighted = *is_highlighted.read(cx);
        let theme = *cx.theme();

        let divider = match axis {
            SplitAxis::Horizontal => div().w(px(BORDER_WIDTH)).h_full(),
            SplitAxis::Vertical => div().h(px(BORDER_WIDTH)).w_full(),
        };
        let interactive = match axis {
            SplitAxis::Horizontal => div()
                .id((id.clone(), "handle"))
                .absolute()
                .left(px(-3.0))
                .top_0()
                .w(px(7.0))
                .h_full()
                .cursor_col_resize(),
            SplitAxis::Vertical => div()
                .id((id.clone(), "handle"))
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
            .bg(if highlighted {
                theme.border_strong
            } else {
                gpui::transparent_black()
            })
            .child(
                interactive
                    .when(enabled, |this| {
                        let hover_state = is_highlighted.clone();
                        let drag_state = is_highlighted.clone();
                        let drop_state = is_highlighted.clone();
                        let key_ctx = keyboard;
                        let drag_id = id.clone();
                        this.on_hover(move |&hovered, _window, cx| {
                            hover_state.write(cx, hovered);
                        })
                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                        .on_drag(
                            DraggedSplitPane { id: drag_id },
                            move |_, _, _window, cx| {
                                drag_state.write(cx, true);
                                cx.new(|_| Empty)
                            },
                        )
                        .on_drop::<DraggedSplitPane>(move |_, _, cx| {
                            drop_state.write(cx, false);
                            cx.stop_propagation();
                        })
                        // Keyboard accessibility: Arrow keys resize, Enter/Escape commit/cancel
                        .tab_index(0)
                        .when_some(key_ctx, |this, ctx| {
                            let resize = ctx.resize.clone();
                            let resize_end = ctx.resize_end.clone();
                            let first_size = ctx.first_size;
                            let min_first = ctx.min_first;
                            let min_second = ctx.min_second;
                            // Use a generous upper bound for simple keyboard clamping;
                            // the SplitPaneState's commit_resize handles the real bounds.
                            let total_estimate = (first_size + min_second + 800.0).max(1200.0);
                            this.on_key_down(move |event: &KeyDownEvent, window, cx| {
                                let key = event.keystroke.key.as_str();
                                match key {
                                    "left" | "up" => {
                                        if let Some(ref handler) = resize {
                                            let next = clamp_split_size(
                                                first_size - KEYBOARD_RESIZE_STEP,
                                                total_estimate,
                                                min_first,
                                                min_second,
                                            );
                                            handler(next, window, cx);
                                            cx.stop_propagation();
                                        }
                                    }
                                    "right" | "down" => {
                                        if let Some(ref handler) = resize {
                                            let next = clamp_split_size(
                                                first_size + KEYBOARD_RESIZE_STEP,
                                                total_estimate,
                                                min_first,
                                                min_second,
                                            );
                                            handler(next, window, cx);
                                            cx.stop_propagation();
                                        }
                                    }
                                    "home" => {
                                        if let Some(ref handler) = resize {
                                            handler(min_first, window, cx);
                                            cx.stop_propagation();
                                        }
                                    }
                                    "end" => {
                                        if let Some(ref handler) = resize {
                                            let end_size = clamp_split_size(
                                                first_size + KEYBOARD_RESIZE_LARGE_STEP,
                                                total_estimate,
                                                min_first,
                                                min_second,
                                            );
                                            handler(end_size, window, cx);
                                            cx.stop_propagation();
                                        }
                                    }
                                    "enter" | "space" => {
                                        if let Some(ref handler) = resize_end {
                                            handler(window, cx);
                                            cx.stop_propagation();
                                        }
                                    }
                                    "escape" => {
                                        if let Some(ref handler) = resize_end {
                                            handler(window, cx);
                                            cx.stop_propagation();
                                        }
                                    }
                                    _ => {}
                                }
                            })
                        })
                    })
                    .hover(move |style| style.bg(gpui::black().opacity(0.08))),
            )
    })
}
