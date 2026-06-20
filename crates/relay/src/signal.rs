use std::{cell::RefCell, rc::Rc};

use gpui::App;

use crate::{ReactiveRuntime, SignalId, init};

/// A read/write reactive value.
///
/// A signal is shallow by design: relay tracks reads and writes to the signal
/// handle, not mutations of fields inside `T`.
pub struct Signal<T> {
    id: SignalId,
    value: Rc<RefCell<T>>,
}

impl<T> Signal<T> {
    /// Create a signal and allocate its runtime identity.
    pub fn new(cx: &mut App, value: T) -> Self {
        init(cx);
        let id = cx.global::<ReactiveRuntime>().allocate_signal();
        Self {
            id,
            value: Rc::new(RefCell::new(value)),
        }
    }

    /// Return this signal's runtime identity.
    pub fn id(&self) -> SignalId {
        self.id
    }

    /// Read with dependency tracking and without cloning.
    pub fn read<R>(&self, cx: &App, f: impl FnOnce(&T) -> R) -> R {
        cx.global::<ReactiveRuntime>().track_signal(self.id);
        f(&self.value.borrow())
    }

    /// Read without dependency tracking and without cloning.
    pub fn peek<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&self.value.borrow())
    }

    /// Set the signal and notify dependents when the value changed.
    pub fn set(&self, cx: &mut App, value: T)
    where
        T: PartialEq,
    {
        let changed = {
            let mut current = self.value.borrow_mut();
            if *current == value {
                false
            } else {
                *current = value;
                true
            }
        };

        if changed {
            let notifications = cx.global::<ReactiveRuntime>().notify_signal(self.id);
            ReactiveRuntime::flush_notifications(cx, notifications);
        }
    }

    /// Mutate the signal in place.
    ///
    /// The closure returns whether dependents should be notified. This keeps the
    /// API useful for non-`PartialEq` values and avoids cloning large snapshots.
    pub fn update(&self, cx: &mut App, update: impl FnOnce(&mut T) -> bool) {
        let changed = update(&mut self.value.borrow_mut());
        if changed {
            let notifications = cx.global::<ReactiveRuntime>().notify_signal(self.id);
            ReactiveRuntime::flush_notifications(cx, notifications);
        }
    }

    /// Split the signal into read and write handles.
    pub fn split(&self) -> (ReadSignal<T>, WriteSignal<T>) {
        (
            ReadSignal {
                signal: self.clone(),
            },
            WriteSignal {
                signal: self.clone(),
            },
        )
    }
}

impl<T: Clone> Signal<T> {
    /// Clone the current value with dependency tracking.
    pub fn get(&self, cx: &App) -> T {
        self.read(cx, Clone::clone)
    }

    /// Clone the current value without dependency tracking.
    pub fn get_untracked(&self) -> T {
        self.peek(Clone::clone)
    }
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            value: self.value.clone(),
        }
    }
}

/// Read-only handle for a [`Signal`].
pub struct ReadSignal<T> {
    signal: Signal<T>,
}

impl<T> ReadSignal<T> {
    /// Return the source signal identity.
    pub fn id(&self) -> SignalId {
        self.signal.id()
    }

    /// Read with dependency tracking and without cloning.
    pub fn read<R>(&self, cx: &App, f: impl FnOnce(&T) -> R) -> R {
        self.signal.read(cx, f)
    }

    /// Read without dependency tracking and without cloning.
    pub fn peek<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.signal.peek(f)
    }
}

impl<T: Clone> ReadSignal<T> {
    /// Clone the current value with dependency tracking.
    pub fn get(&self, cx: &App) -> T {
        self.signal.get(cx)
    }

    /// Clone the current value without dependency tracking.
    pub fn get_untracked(&self) -> T {
        self.signal.get_untracked()
    }
}

impl<T> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal.clone(),
        }
    }
}

/// Write-only handle for a [`Signal`].
pub struct WriteSignal<T> {
    signal: Signal<T>,
}

impl<T> WriteSignal<T> {
    /// Return the source signal identity.
    pub fn id(&self) -> SignalId {
        self.signal.id()
    }

    /// Set the signal and notify dependents when the value changed.
    pub fn set(&self, cx: &mut App, value: T)
    where
        T: PartialEq,
    {
        self.signal.set(cx, value);
    }

    /// Mutate the signal in place.
    pub fn update(&self, cx: &mut App, update: impl FnOnce(&mut T) -> bool) {
        self.signal.update(cx, update);
    }
}

impl<T> Clone for WriteSignal<T> {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal.clone(),
        }
    }
}
