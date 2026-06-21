//! Reactive vector helpers for [`Signal<Vec<T>>`].
//!
//! These methods let callers mutate a signal-backed list in place while still
//! going through the normal signal notification path. They are thin wrappers
//! around [`Signal::update`](crate::Signal::update) that report `true` so
//! dependents are always notified after a structural change — list diffs are
//! rarely cheap to compare with `PartialEq`, and callers usually want every
//! push/remove to refresh.

use gpui::App;

use crate::Signal;

/// Extension trait with list mutation helpers for `Signal<Vec<T>>`.
pub trait SignalVecExt<T: 'static> {
    /// Push a value onto the end of the list and notify dependents.
    fn push(&self, cx: &mut App, value: T);

    /// Insert a value at `index`, shifting later elements right.
    fn insert(&self, cx: &mut App, index: usize, value: T);

    /// Remove the value at `index` and return it.
    fn remove(&self, cx: &mut App, index: usize) -> Option<T>;

    /// Remove the first value matching `predicate` and return whether one was
    /// removed.
    fn remove_first(&self, cx: &mut App, predicate: impl FnMut(&T) -> bool) -> bool;

    /// Retain only values for which `predicate` returns true.
    fn retain(&self, cx: &mut App, predicate: impl FnMut(&T) -> bool);

    /// Clear all values.
    fn clear(&self, cx: &mut App);

    /// Replace the entire list.
    fn set_all(&self, cx: &mut App, values: Vec<T>);
}

impl<T: 'static> SignalVecExt<T> for Signal<Vec<T>> {
    fn push(&self, cx: &mut App, value: T) {
        self.update(cx, |list| {
            list.push(value);
            true
        });
    }

    fn insert(&self, cx: &mut App, index: usize, value: T) {
        self.update(cx, |list| {
            list.insert(index, value);
            true
        });
    }

    fn remove(&self, cx: &mut App, index: usize) -> Option<T> {
        let mut removed = None;
        self.update(cx, |list| {
            if index < list.len() {
                removed = Some(list.remove(index));
                true
            } else {
                false
            }
        });
        removed
    }

    fn remove_first(&self, cx: &mut App, mut predicate: impl FnMut(&T) -> bool) -> bool {
        let mut removed = false;
        self.update(cx, |list| {
            if let Some(pos) = list.iter().position(&mut predicate) {
                list.remove(pos);
                removed = true;
                true
            } else {
                false
            }
        });
        removed
    }

    fn retain(&self, cx: &mut App, mut predicate: impl FnMut(&T) -> bool) {
        self.update(cx, |list| {
            let before = list.len();
            list.retain(&mut predicate);
            list.len() != before
        });
    }

    fn clear(&self, cx: &mut App) {
        self.update(cx, |list| {
            if list.is_empty() {
                false
            } else {
                list.clear();
                true
            }
        });
    }

    fn set_all(&self, cx: &mut App, values: Vec<T>) {
        self.update(cx, |list| {
            *list = values;
            true
        });
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{Context, IntoElement, ParentElement, Render, TestApp, Window, div};

    use crate::{ReactiveAppExt, Signal, SignalVecExt, effect, init, track};

    struct ListView {
        items: Signal<Vec<i32>>,
    }

    impl ListView {
        fn new(cx: &mut Context<Self>) -> Self {
            init(cx);
            Self {
                items: cx.signal(Vec::new()),
            }
        }
    }

    impl Render for ListView {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            track(cx, |cx| div().child(self.items.get(cx).len().to_string()))
        }
    }

    #[test]
    fn push_notifies_rendering_entity() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| ListView::new(cx));
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

        app.update_entity(&root, |view, cx| {
            view.items.push(cx, 1);
        });

        assert_eq!(notifications.get(), 1);
        app.update_entity(&root, |view, _cx| {
            assert_eq!(view.items.get_untracked(), vec![1]);
        });
    }

    #[test]
    fn remove_returns_removed_value() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| ListView::new(cx));
        let root = window.root();
        window.draw();

        app.update_entity(&root, |view, cx| {
            view.items.push(cx, 10);
            view.items.push(cx, 20);
            view.items.push(cx, 30);
        });

        let removed = app.update_entity(&root, |view, cx| view.items.remove(cx, 1));
        assert_eq!(removed, Some(20));

        app.update_entity(&root, |view, _cx| {
            assert_eq!(view.items.get_untracked(), vec![10, 30]);
        });
    }

    #[test]
    fn remove_out_of_bounds_returns_none_and_does_not_notify() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| ListView::new(cx));
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

        let removed = app.update_entity(&root, |view, cx| view.items.remove(cx, 5));
        assert_eq!(removed, None);
        assert_eq!(notifications.get(), 0);
    }

    #[test]
    fn retain_filters_values() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| ListView::new(cx));
        let root = window.root();
        window.draw();

        app.update_entity(&root, |view, cx| {
            view.items.push(cx, 1);
            view.items.push(cx, 2);
            view.items.push(cx, 3);
            view.items.push(cx, 4);
            view.items.retain(cx, |v| *v % 2 == 0);
        });

        app.update_entity(&root, |view, _cx| {
            assert_eq!(view.items.get_untracked(), vec![2, 4]);
        });
    }

    #[test]
    fn clear_on_empty_does_not_notify() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| ListView::new(cx));
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

        app.update_entity(&root, |view, cx| {
            view.items.clear(cx);
        });

        assert_eq!(notifications.get(), 0);
    }

    #[test]
    fn effect_reruns_when_list_changes() {
        let mut app = TestApp::new();
        let signal = app.update(|cx| {
            init(cx);
            let signal: Signal<Vec<i32>> = cx.signal(Vec::new());
            signal
        });

        let lengths = Rc::new(Cell::new(0));
        let _effect = app.update({
            let lengths = lengths.clone();
            let signal = signal.clone();
            move |cx| {
                effect(cx, move |cx| {
                    lengths.set(signal.get(cx).len());
                })
            }
        });

        assert_eq!(lengths.get(), 0);

        app.update(|cx| signal.push(cx, 1));
        assert_eq!(lengths.get(), 1);

        app.update(|cx| signal.push(cx, 2));
        assert_eq!(lengths.get(), 2);

        app.update(|cx| signal.remove(cx, 0));
        assert_eq!(lengths.get(), 1);
    }
}
