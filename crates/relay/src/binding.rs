use gpui::App;

use crate::Signal;

/// A UI-agnostic two-way binding around a signal.
///
/// UI crates can adapt this type to concrete controls without making `relay`
/// depend on those controls.
pub struct Binding<T> {
    signal: Signal<T>,
}

impl<T> Binding<T> {
    /// Create a binding from a signal.
    pub fn new(signal: Signal<T>) -> Self {
        Self { signal }
    }

    /// Return the underlying signal.
    pub fn signal(&self) -> &Signal<T> {
        &self.signal
    }

    /// Read with dependency tracking and without cloning.
    pub fn read<R>(&self, cx: &App, f: impl FnOnce(&T) -> R) -> R {
        self.signal.read(cx, f)
    }

    /// Mutate the bound value.
    pub fn update(&self, cx: &mut App, update: impl FnOnce(&mut T) -> bool) {
        self.signal.update(cx, update);
    }

    /// Mutate the bound value without notifying dependents.
    ///
    /// See [`crate::Signal::update_silent`] for when to use this.
    pub fn update_silent(&self, f: impl FnOnce(&mut T)) {
        self.signal.update_silent(f);
    }

    /// Set the bound value without notifying dependents.
    ///
    /// See [`crate::Signal::update_silent`] for when to use this.
    pub fn set_silent(&self, value: T) {
        self.signal.set_silent(value);
    }

    /// Set the bound value and notify dependents when it changed.
    pub fn set(&self, cx: &mut App, value: T)
    where
        T: PartialEq,
    {
        self.signal.set(cx, value);
    }
}

impl<T: Clone> Binding<T> {
    /// Clone the bound value with dependency tracking.
    pub fn get(&self, cx: &App) -> T {
        self.signal.get(cx)
    }
}

impl<T> Clone for Binding<T> {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal.clone(),
        }
    }
}

impl<T> From<Signal<T>> for Binding<T> {
    fn from(signal: Signal<T>) -> Self {
        Self::new(signal)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::TestApp;

    use crate::{ReactiveAppExt, Signal, init};

    use super::*;

    #[test]
    fn binding_sets_and_reads_signal_value() {
        let mut app = TestApp::new();
        let binding = app.update(|cx| {
            init(cx);
            Binding::from(Signal::new(cx, false))
        });

        app.update(|cx| {
            binding.set(cx, true);
        });

        app.read(|cx| {
            assert!(binding.get(cx));
        });
    }

    #[test]
    fn binding_clone_shares_underlying_signal() {
        let mut app = TestApp::new();
        let binding = app.update(|cx| {
            init(cx);
            cx.binding(10)
        });

        let cloned = binding.clone();

        // Write via the clone, read via the original.
        app.update(|cx| cloned.set(cx, 20));
        app.read(|cx| assert_eq!(binding.get(cx), 20));

        // Write via the original, read via the clone.
        app.update(|cx| binding.set(cx, 30));
        app.read(|cx| assert_eq!(cloned.get(cx), 30));
    }

    #[test]
    fn binding_update_silent_does_not_notify() {
        let mut app = TestApp::new();
        let binding = app.update(|cx| {
            init(cx);
            cx.binding(0)
        });

        let runs = Rc::new(Cell::new(0));
        let _effect = app.update({
            let runs = runs.clone();
            let binding = binding.clone();
            move |cx| {
                crate::effect(cx, move |cx| {
                    let _ = binding.get(cx);
                    runs.set(runs.get() + 1);
                })
            }
        });
        assert_eq!(runs.get(), 1);

        // Silent update — effect should NOT rerun.
        app.update(|_cx| binding.update_silent(|v| *v = 42));
        assert_eq!(runs.get(), 1);

        // But the value was changed.
        app.read(|cx| assert_eq!(binding.get(cx), 42));
    }

    #[test]
    fn binding_from_signal_preserves_identity() {
        let mut app = TestApp::new();
        let (signal_id, binding) = app.update(|cx| {
            init(cx);
            let signal = Signal::new(cx, "hello".into());
            let id = signal.id();
            let binding: Binding<String> = signal.into();
            (id, binding)
        });

        assert_eq!(binding.signal().id(), signal_id);
    }
}
