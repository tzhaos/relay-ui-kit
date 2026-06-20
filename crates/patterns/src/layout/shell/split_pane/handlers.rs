use std::rc::Rc;

use gpui::Entity;
use relay_ui_core::interaction::{SharedChangeHandler, SharedDismissHandler};

use super::state::SplitPaneState;

pub(super) fn resize_handler(
    state: Option<Entity<SplitPaneState>>,
    external: Option<SharedChangeHandler<f32>>,
) -> Option<SharedChangeHandler<f32>> {
    if state.is_none() && external.is_none() {
        return None;
    }

    Some(Rc::new(move |next, window, cx| {
        let mut changed = state.is_none();
        if let Some(state) = &state {
            state.update(cx, |state, cx| {
                if state.preview_resize_to(next) {
                    changed = true;
                    cx.notify();
                }
            });
        }
        if let (true, Some(external)) = (changed, external.as_ref()) {
            external(next, window, cx);
        }
    }))
}

pub(super) fn resize_end_handler(
    state: Option<Entity<SplitPaneState>>,
    external: Option<SharedDismissHandler>,
) -> Option<SharedDismissHandler> {
    if state.is_none() && external.is_none() {
        return None;
    }

    Some(Rc::new(move |window, cx| {
        let mut changed = state.is_none();
        if let Some(state) = &state {
            state.update(cx, |state, cx| {
                if state.commit_resize() {
                    changed = true;
                    cx.notify();
                }
            });
        }
        if let (true, Some(external)) = (changed, external.as_ref()) {
            external(window, cx);
        }
    }))
}
