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
    use gpui::TestApp;

    use crate::{Signal, init};

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
}
