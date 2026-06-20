use gpui::{App, ElementId, Window};

use crate::{Binding, Signal};

/// Convenience methods for creating relay state from GPUI element state.
pub trait WindowSignalExt {
    /// Use a signal that lives as long as the keyed element state is retained.
    fn use_signal<T: 'static>(
        &mut self,
        key: impl Into<ElementId>,
        cx: &mut App,
        init: impl FnOnce() -> T,
    ) -> Signal<T>;

    /// Use a binding backed by keyed element state.
    fn use_binding<T: 'static>(
        &mut self,
        key: impl Into<ElementId>,
        cx: &mut App,
        init: impl FnOnce() -> T,
    ) -> Binding<T>;
}

impl WindowSignalExt for Window {
    fn use_signal<T: 'static>(
        &mut self,
        key: impl Into<ElementId>,
        cx: &mut App,
        init: impl FnOnce() -> T,
    ) -> Signal<T> {
        let state = self.use_keyed_state(key, cx, |_, cx| Signal::new(cx, init()));
        state.read(cx).clone()
    }

    fn use_binding<T: 'static>(
        &mut self,
        key: impl Into<ElementId>,
        cx: &mut App,
        init: impl FnOnce() -> T,
    ) -> Binding<T> {
        Binding::from(self.use_signal(key, cx, init))
    }
}
