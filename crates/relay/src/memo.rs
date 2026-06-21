use gpui::App;

use crate::{Effect, Signal, effect};

/// A signal-backed derived value.
///
/// `Memo` tracks the signals read by its compute function and updates its
/// internal signal when those dependencies change.
pub struct Memo<T> {
    signal: Signal<T>,
    _effect: Effect,
}

impl<T> Memo<T>
where
    T: PartialEq + 'static,
{
    /// Create a memo from a compute function.
    pub fn new(cx: &mut App, compute: impl Fn(&App) -> T + 'static) -> Self {
        let initial = compute(cx);
        let signal = Signal::new(cx, initial);
        let signal_for_effect = signal.clone();
        let effect = effect(cx, move |cx| {
            signal_for_effect.set(cx, compute(cx));
        });

        Self {
            signal,
            _effect: effect,
        }
    }

    /// Return this memo's signal.
    pub fn signal(&self) -> &Signal<T> {
        &self.signal
    }

    /// Read the memo with dependency tracking and without cloning.
    pub fn read<R>(&self, cx: &App, f: impl FnOnce(&T) -> R) -> R {
        self.signal.read(cx, f)
    }
}

impl<T> Memo<T>
where
    T: Clone + PartialEq + 'static,
{
    /// Clone the memo value with dependency tracking.
    pub fn get(&self, cx: &App) -> T {
        self.signal.get(cx)
    }
}

impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal.clone(),
            // Effect holds only an id; cloning is safe — the underlying effect
            // is shared, not duplicated.
            _effect: Effect::from_id(self._effect.id()),
        }
    }
}
