use gpui::{
    App, AppContext as _, Empty, InteractiveElement, MouseButton, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use relay_ui_primitives::theme::ActiveTheme;

use super::{SplitAxis, drag::DraggedSplitPane};

pub(super) fn render_split_handle(
    id: &'static str,
    axis: SplitAxis,
    enabled: bool,
    window: &mut Window,
    cx: &mut App,
) -> gpui::Div {
    window.with_id((id, 0usize), |window| {
        let is_highlighted = window.use_state(cx, |_window, _cx| false);
        let highlighted = *is_highlighted.read(cx);
        let theme = *cx.theme();

        let divider = match axis {
            SplitAxis::Horizontal => div().w(px(1.0)).h_full(),
            SplitAxis::Vertical => div().h(px(1.0)).w_full(),
        };
        let interactive = match axis {
            SplitAxis::Horizontal => div()
                .id((id, 1usize))
                .absolute()
                .left(px(-3.0))
                .top_0()
                .w(px(7.0))
                .h_full()
                .cursor_col_resize(),
            SplitAxis::Vertical => div()
                .id((id, 1usize))
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
                        this.on_hover(move |&hovered, _window, cx| {
                            hover_state.write(cx, hovered);
                        })
                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                        .on_drag(DraggedSplitPane { id }, move |_, _, _window, cx| {
                            drag_state.write(cx, true);
                            cx.new(|_| Empty)
                        })
                        .on_drop::<DraggedSplitPane>(move |_, _, cx| {
                            drop_state.write(cx, false);
                            cx.stop_propagation();
                        })
                    })
                    .hover(move |style| style.bg(gpui::black().opacity(0.08))),
            )
    })
}
