use gpui::{App, Context, FocusHandle, Subscription, Window};

use crate::Signal;

/// Entity-owned focus state that mirrors a GPUI [`FocusHandle`] into relay
/// signal state.
///
/// This is intentionally a thin coordination wrapper around GPUI focus APIs:
/// the focus handle remains the source of truth, while the signal provides a
/// composable way to derive and observe focus-dependent state.
pub struct FocusState {
    handle: FocusHandle,
    focused: Signal<bool>,
    subscriptions: Vec<Subscription>,
    attached: bool,
}

/// Create a focus state for the current entity.
pub fn use_focus_state<T: 'static>(cx: &mut Context<T>) -> FocusState {
    FocusState::new(cx)
}

impl FocusState {
    /// Create a focus state for the current entity.
    pub fn new<T: 'static>(cx: &mut Context<T>) -> Self {
        Self {
            handle: cx.focus_handle(),
            focused: Signal::new(cx, false),
            subscriptions: Vec::new(),
            attached: false,
        }
    }

    /// Return the underlying GPUI focus handle.
    pub fn handle(&self) -> &FocusHandle {
        &self.handle
    }

    /// Return the signal that mirrors whether this focus scope currently contains focus.
    pub fn focused(&self) -> &Signal<bool> {
        &self.focused
    }

    /// Return whether this focus scope currently contains focus.
    pub fn is_focused(&self, cx: &App) -> bool {
        self.focused.get(cx)
    }

    /// Attach focus listeners to the current window.
    ///
    /// This is idempotent. Call it once when the entity has access to a window,
    /// typically from the window-aware constructor used by `open_window`.
    pub fn attach<T: 'static>(&mut self, window: &mut Window, cx: &mut Context<T>) {
        if self.attached {
            return;
        }

        let has_focus = self.handle.contains_focused(window, cx);
        self.focused.set(cx, has_focus);

        let focused_for_focus_in = self.focused.clone();
        let on_focus_in = cx.on_focus_in(&self.handle, window, move |_, _window, cx| {
            focused_for_focus_in.set(cx, true);
        });

        let focused_for_focus_out = self.focused.clone();
        let on_focus_out = cx.on_focus_out(&self.handle, window, move |_, _event, _window, cx| {
            focused_for_focus_out.set(cx, false);
        });

        self.subscriptions.push(on_focus_in);
        self.subscriptions.push(on_focus_out);
        self.attached = true;
    }

    /// Drop all focus listeners.
    pub fn detach(&mut self) {
        self.subscriptions.clear();
        self.attached = false;
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Context, InteractiveElement, IntoElement, Render, TestApp, Window, div};

    use crate::init;

    use super::*;

    struct FocusView {
        focus: FocusState,
    }

    impl FocusView {
        fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
            init(cx);
            let mut focus = use_focus_state(cx);
            focus.attach(window, cx);
            Self { focus }
        }
    }

    impl Render for FocusView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().track_focus(self.focus.handle())
        }
    }

    #[test]
    fn focus_state_tracks_window_focus_changes() {
        let mut app = TestApp::new();
        let mut window = app.open_window(FocusView::new);
        window.draw();

        let handle = window.read(|view, _cx| view.focus.handle().clone());

        window.read(|view, cx| {
            assert!(!view.focus.is_focused(cx));
        });

        window.update(|_view, window, cx| {
            window.activate_window();
            window.focus(&handle, cx);
        });
        window.draw();

        window.read(|view, cx| {
            assert!(view.focus.is_focused(cx));
        });

        window.update(|_view, window, cx| {
            window.activate_window();
            window.focus(&cx.focus_handle(), cx);
        });
        window.draw();

        window.read(|view, cx| {
            assert!(!view.focus.is_focused(cx));
        });
    }
}
