use std::rc::Rc;

use gpui::{App, Entity, Window};

use super::state::SplitPaneState;

pub(super) type ResizeHandler = Rc<dyn Fn(f32, &mut Window, &mut App) + 'static>;
pub(super) type ResizeEndHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

pub(super) fn resize_handler(
    state: Option<Entity<SplitPaneState>>,
    external: Option<ResizeHandler>,
) -> Option<ResizeHandler> {
    if state.is_none() && external.is_none() {
        return None;
    }

    Some(Rc::new(move |next, window, cx| {
        if let Some(state) = &state {
            state.update(cx, |state, cx| {
                if state.preview_resize_to(next) {
                    cx.notify();
                }
            });
        }
        if let Some(external) = &external {
            external(next, window, cx);
        }
    }))
}

pub(super) fn resize_end_handler(
    state: Option<Entity<SplitPaneState>>,
    external: Option<ResizeEndHandler>,
) -> Option<ResizeEndHandler> {
    if state.is_none() && external.is_none() {
        return None;
    }

    Some(Rc::new(move |window, cx| {
        if let Some(state) = &state {
            state.update(cx, |state, cx| {
                if state.commit_resize() {
                    cx.notify();
                }
            });
        }
        if let Some(external) = &external {
            external(window, cx);
        }
    }))
}
