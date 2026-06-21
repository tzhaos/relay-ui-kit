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

    /// Mutate the signal without notifying dependents.
    ///
    /// Use this when an effect needs to write back to a signal it also reads,
    /// or when coordinating internal state that should not trigger a refresh on
    /// its own. Prefer [`Signal::update`] for ordinary mutations.
    pub fn update_silent(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.value.borrow_mut());
    }

    /// Set the signal value without notifying dependents.
    ///
    /// See [`Signal::update_silent`] for when to use this.
    pub fn set_silent(&self, value: T) {
        *self.value.borrow_mut() = value;
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

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{Context, IntoElement, ParentElement, Render, TestApp, Window, div};

    use crate::{Signal, effect, init, track};

    struct SilentView {
        signal: Signal<i32>,
    }

    impl SilentView {
        fn new(cx: &mut Context<Self>) -> Self {
            init(cx);
            Self {
                signal: Signal::new(cx, 0),
            }
        }
    }

    impl Render for SilentView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            track(cx, |cx| div().child(self.signal.get(cx).to_string()))
        }
    }

    #[test]
    fn update_silent_does_not_notify() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| SilentView::new(cx));
        let root = window.root();
        window.draw();

        let notifications = Rc::new(Cell::new(0));
        let _subscription = app.update({
            let notifications = notifications.clone();
            let root = root.clone();
            move |cx| {
                cx.observe(&root, move |_, _| {
                    notifications.set(notifications.get() + 1);
                })
            }
        });

        app.update_entity(&root, |view, _cx| {
            view.signal.update_silent(|v| *v = 42);
        });

        assert_eq!(notifications.get(), 0);
        app.update_entity(&root, |view, _cx| {
            assert_eq!(view.signal.get_untracked(), 42);
        });
    }

    #[test]
    fn set_silent_does_not_notify() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| SilentView::new(cx));
        let root = window.root();
        window.draw();

        let notifications = Rc::new(Cell::new(0));
        let _subscription = app.update({
            let notifications = notifications.clone();
            let root = root.clone();
            move |cx| {
                cx.observe(&root, move |_, _| {
                    notifications.set(notifications.get() + 1);
                })
            }
        });

        app.update_entity(&root, |view, _cx| {
            view.signal.set_silent(7);
        });

        assert_eq!(notifications.get(), 0);
        app.update_entity(&root, |view, _cx| {
            assert_eq!(view.signal.get_untracked(), 7);
        });
    }

    #[test]
    fn update_silent_then_visible_update_notifies() {
        let mut app = TestApp::new();
        let signal = app.update(|cx| {
            init(cx);
            Signal::new(cx, 0)
        });

        let runs = Rc::new(Cell::new(0));
        let _effect = app.update({
            let runs = runs.clone();
            let signal = signal.clone();
            move |cx| {
                effect(cx, move |cx| {
                    signal.get(cx);
                    runs.set(runs.get() + 1);
                })
            }
        });

        assert_eq!(runs.get(), 1);

        app.update(|_cx| signal.update_silent(|v| *v = 1));
        assert_eq!(runs.get(), 1);

        app.update(|cx| signal.set(cx, 2));
        assert_eq!(runs.get(), 2);
    }

    // --- split read/write isolation ---

    #[test]
    fn split_produces_isolated_read_and_write_handles() {
        let mut app = TestApp::new();
        let signal = app.update(|cx| {
            init(cx);
            Signal::new(cx, 10)
        });
        let (read, write) = signal.split();

        // Write handle can set the value.
        app.update(|cx| write.set(cx, 20));
        app.read(|cx| assert_eq!(read.get(cx), 20));
    }

    #[test]
    fn read_handle_tracks_dependencies() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| SilentView::new(cx));
        let root = window.root();
        window.draw();

        // Replace the view's signal with a split read handle to verify
        // tracking works the same way.
        let signal = app.update_entity(&root, |view, _cx| view.signal.clone());
        let (read, _write) = signal.split();

        let notifications = Rc::new(Cell::new(0));
        let _sub = app.update({
            let notifications = notifications.clone();
            let root = root.clone();
            move |cx| {
                cx.observe(&root, move |_, _| {
                    notifications.set(notifications.get() + 1);
                })
            }
        });

        // Reading the read handle in a tracked context subscribes.
        app.update_entity(&root, |_, cx| {
            track(cx, |cx| {
                let _ = read.get(cx);
            });
        });

        // Writing via the write handle should notify.
        app.update_entity(&root, |view, cx| {
            view.signal.set(cx, 99);
        });
        assert_eq!(notifications.get(), 1);
    }

    // --- update with false return does not notify ---

    #[test]
    fn update_returning_false_does_not_notify() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| SilentView::new(cx));
        let root = window.root();
        window.draw();

        let notifications = Rc::new(Cell::new(0));
        let _sub = app.update({
            let notifications = notifications.clone();
            let root = root.clone();
            move |cx| {
                cx.observe(&root, move |_, _| {
                    notifications.set(notifications.get() + 1);
                })
            }
        });

        // update closure returns false → no notification.
        app.update_entity(&root, |view, cx| {
            view.signal.update(cx, |_v| false);
        });
        assert_eq!(notifications.get(), 0);

        // update closure returns true → notification.
        app.update_entity(&root, |view, cx| {
            view.signal.update(cx, |v| {
                *v += 1;
                true
            });
        });
        assert_eq!(notifications.get(), 1);
    }

    // --- peek reads without tracking ---

    #[test]
    fn peek_reads_value_without_tracking() {
        let mut app = TestApp::new();
        let signal = app.update(|cx| {
            init(cx);
            Signal::new(cx, 42)
        });

        // peek inside an effect — should NOT register as a dependency.
        let runs = Rc::new(Cell::new(0));
        let _effect = app.update({
            let runs = runs.clone();
            let signal = signal.clone();
            move |cx| {
                effect(cx, move |_cx| {
                    let _ = signal.peek(|v| *v);
                    runs.set(runs.get() + 1);
                })
            }
        });
        assert_eq!(runs.get(), 1);

        // Signal change should NOT trigger the effect (peek doesn't track).
        app.update(|cx| signal.set(cx, 100));
        assert_eq!(runs.get(), 1, "peek should not register dependency");
    }

    // --- set with same value on non-PartialEq type via update ---

    #[test]
    fn update_works_for_non_partial_eq_types() {
        // A type that doesn't implement PartialEq.
        struct NonEq(i32);

        let mut app = TestApp::new();
        let signal = app.update(|cx| {
            init(cx);
            Signal::new(cx, NonEq(0))
        });

        // set() requires PartialEq, but update() works for any type.
        app.update(|cx| {
            signal.update(cx, |v| {
                v.0 = 5;
                true
            });
        });

        app.read(|cx| {
            signal.read(cx, |v| assert_eq!(v.0, 5));
        });
    }
}
