//! Keyed selection primitive.

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    hash::Hash,
    rc::Rc,
};

use gpui::App;

use crate::{Signal, batch};

/// Fine-grained keyed selection state.
///
/// `Selector<K>` is useful for list-shaped UIs where each row only needs to
/// know whether its own key is selected. Reading [`Selector::is_selected`]
/// tracks a per-key signal. Updating the selected key only notifies the
/// previously selected and newly selected keys when those keys are registered.
pub struct Selector<K> {
    selected: Signal<Option<K>>,
    keyed: Rc<RefCell<HashMap<K, Signal<bool>>>>,
}

impl<K> Selector<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    /// Create a selector with the current selected key.
    pub fn new(cx: &mut App, selected: Option<K>) -> Self {
        Self {
            selected: Signal::new(cx, selected),
            keyed: Rc::default(),
        }
    }

    /// Return the underlying selected-key signal.
    pub fn selected_signal(&self) -> &Signal<Option<K>> {
        &self.selected
    }

    /// Clone the selected key with dependency tracking.
    pub fn get(&self, cx: &App) -> Option<K> {
        self.selected.get(cx)
    }

    /// Clone the selected key without dependency tracking.
    pub fn get_untracked(&self) -> Option<K> {
        self.selected.get_untracked()
    }

    /// Return whether `key` is selected, tracking only this key.
    pub fn is_selected(&self, cx: &mut App, key: K) -> bool {
        self.key_signal(cx, key).get(cx)
    }

    /// Select a key.
    pub fn select(&self, cx: &mut App, key: K) {
        self.set(cx, Some(key));
    }

    /// Clear the current selection.
    pub fn clear(&self, cx: &mut App) {
        self.set(cx, None);
    }

    /// Set the selected key.
    pub fn set(&self, cx: &mut App, selected: Option<K>) {
        let previous = self.selected.get_untracked();
        if previous == selected {
            return;
        }

        batch(cx, |cx| {
            self.selected.set(cx, selected.clone());

            let keyed = self.keyed.borrow();
            if let Some(previous) = previous
                && let Some(signal) = keyed.get(&previous)
            {
                signal.set(cx, false);
            }
            if let Some(selected) = selected
                && let Some(signal) = keyed.get(&selected)
            {
                signal.set(cx, true);
            }
        });
    }

    /// Drop per-key signals for keys that are no longer relevant.
    pub fn retain_keys(&self, keys: impl IntoIterator<Item = K>) {
        let keys = keys.into_iter().collect::<HashSet<_>>();
        self.keyed.borrow_mut().retain(|key, _| keys.contains(key));
    }

    /// Reconcile this selector with the currently available keys.
    ///
    /// This drops per-key signals for removed keys and clears the selected key
    /// if it is no longer present. Returns whether the selection was cleared.
    pub fn reconcile_keys(&self, cx: &mut App, keys: impl IntoIterator<Item = K>) -> bool {
        let keys = keys.into_iter().collect::<HashSet<_>>();
        let should_clear = self
            .selected
            .peek(|selected| selected.as_ref().is_some_and(|key| !keys.contains(key)));

        if should_clear {
            self.clear(cx);
        }

        self.keyed.borrow_mut().retain(|key, _| keys.contains(key));
        should_clear
    }

    /// Drop all per-key signals. The selected key itself is unchanged.
    pub fn clear_keyed_signals(&self) {
        self.keyed.borrow_mut().clear();
    }

    fn key_signal(&self, cx: &mut App, key: K) -> Signal<bool> {
        if let Some(signal) = self.keyed.borrow().get(&key) {
            return signal.clone();
        }

        let is_selected = self.selected.get_untracked().as_ref() == Some(&key);
        let signal = Signal::new(cx, is_selected);
        self.keyed.borrow_mut().insert(key, signal.clone());
        signal
    }
}

impl<K> Clone for Selector<K> {
    fn clone(&self) -> Self {
        Self {
            selected: self.selected.clone(),
            keyed: self.keyed.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::TestApp;

    use crate::{Selector, effect, init};

    #[test]
    fn selector_notifies_only_previous_and_next_key_effects() {
        let mut app = TestApp::new();
        let selector = app.update(|cx| {
            init(cx);
            Selector::new(cx, Some(1_u64))
        });

        let one_runs = Rc::new(Cell::new(0));
        let two_runs = Rc::new(Cell::new(0));
        let three_runs = Rc::new(Cell::new(0));

        let _effects = app.update({
            let selector = selector.clone();
            let one_runs = one_runs.clone();
            let two_runs = two_runs.clone();
            let three_runs = three_runs.clone();
            move |cx| {
                let one = effect(cx, {
                    let selector = selector.clone();
                    move |cx| {
                        let _ = selector.is_selected(cx, 1);
                        one_runs.set(one_runs.get() + 1);
                    }
                });
                let two = effect(cx, {
                    let selector = selector.clone();
                    move |cx| {
                        let _ = selector.is_selected(cx, 2);
                        two_runs.set(two_runs.get() + 1);
                    }
                });
                let three = effect(cx, {
                    let selector = selector.clone();
                    move |cx| {
                        let _ = selector.is_selected(cx, 3);
                        three_runs.set(three_runs.get() + 1);
                    }
                });
                (one, two, three)
            }
        });

        assert_eq!(one_runs.get(), 1);
        assert_eq!(two_runs.get(), 1);
        assert_eq!(three_runs.get(), 1);

        app.update(|cx| selector.select(cx, 2));

        assert_eq!(one_runs.get(), 2);
        assert_eq!(two_runs.get(), 2);
        assert_eq!(three_runs.get(), 1);
    }

    #[test]
    fn selector_get_tracks_whole_selected_key() {
        let mut app = TestApp::new();
        let selector = app.update(|cx| {
            init(cx);
            Selector::new(cx, Some("first"))
        });

        let runs = Rc::new(Cell::new(0));
        let _effect = app.update({
            let selector = selector.clone();
            let runs = runs.clone();
            move |cx| {
                effect(cx, move |cx| {
                    let _ = selector.get(cx);
                    runs.set(runs.get() + 1);
                })
            }
        });

        assert_eq!(runs.get(), 1);

        app.update(|cx| selector.select(cx, "second"));

        assert_eq!(runs.get(), 2);
    }

    #[test]
    fn selector_reconcile_keys_clears_missing_selection() {
        let mut app = TestApp::new();
        let selector = app.update(|cx| {
            init(cx);
            Selector::new(cx, Some(2_u64))
        });

        let changed = app.update(|cx| selector.reconcile_keys(cx, [1_u64, 3]));

        assert!(changed);
        assert_eq!(selector.get_untracked(), None);
    }

    #[test]
    fn selector_reconcile_keys_keeps_available_selection() {
        let mut app = TestApp::new();
        let selector = app.update(|cx| {
            init(cx);
            Selector::new(cx, Some(2_u64))
        });

        let changed = app.update(|cx| selector.reconcile_keys(cx, [1_u64, 2, 3]));

        assert!(!changed);
        assert_eq!(selector.get_untracked(), Some(2));
    }

    #[test]
    fn selector_reconcile_keys_prunes_keyed_signals() {
        let mut app = TestApp::new();
        let selector = app.update(|cx| {
            init(cx);
            Selector::new(cx, Some(2_u64))
        });

        app.update(|cx| {
            selector.is_selected(cx, 1);
            selector.is_selected(cx, 2);
            selector.is_selected(cx, 3);
            selector.reconcile_keys(cx, [2_u64]);
        });

        assert_eq!(selector.keyed.borrow().len(), 1);
        assert!(selector.keyed.borrow().contains_key(&2));
    }

    #[test]
    fn selector_reconcile_keys_notifies_removed_selected_key() {
        let mut app = TestApp::new();
        let selector = app.update(|cx| {
            init(cx);
            Selector::new(cx, Some(2_u64))
        });

        let runs = Rc::new(Cell::new(0));
        let _effect = app.update({
            let selector = selector.clone();
            let runs = runs.clone();
            move |cx| {
                effect(cx, move |cx| {
                    let _ = selector.is_selected(cx, 2);
                    runs.set(runs.get() + 1);
                })
            }
        });

        assert_eq!(runs.get(), 1);

        app.update(|cx| {
            selector.reconcile_keys(cx, [1_u64]);
        });

        assert_eq!(runs.get(), 2);
    }
}
