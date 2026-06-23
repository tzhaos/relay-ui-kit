use std::hash::Hash;

use gpui::App;

use crate::{Memo, SelectedItemExt, Selector, Signal};

/// A higher-level single-selection model built on top of [`Selector`].
pub struct SelectionModel<K> {
    selector: Selector<K>,
    has_selection: Memo<bool>,
}

/// Create a selection model with the given initial key.
pub fn use_selection_model<K>(cx: &mut App, selected: Option<K>) -> SelectionModel<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    SelectionModel::new(cx, selected)
}

impl<K> SelectionModel<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    /// Create a selection model with the given initial key.
    pub fn new(cx: &mut App, selected: Option<K>) -> Self {
        let selector = Selector::new(cx, selected);
        let selector_for_has_selection = selector.clone();

        Self {
            selector,
            has_selection: Memo::new(cx, move |cx| selector_for_has_selection.get(cx).is_some()),
        }
    }

    /// Return the underlying selector.
    pub fn selector(&self) -> &Selector<K> {
        &self.selector
    }

    /// Return a memo that is `true` when any key is selected.
    pub fn has_selection(&self) -> &Memo<bool> {
        &self.has_selection
    }

    /// Clone the selected key with dependency tracking.
    pub fn get(&self, cx: &App) -> Option<K> {
        self.selector.get(cx)
    }

    /// Clone the selected key without dependency tracking.
    pub fn get_untracked(&self) -> Option<K> {
        self.selector.get_untracked()
    }

    /// Return whether the given key is selected.
    pub fn is_selected(&self, cx: &mut App, key: K) -> bool {
        self.selector.is_selected(cx, key)
    }

    /// Select a key.
    pub fn select(&self, cx: &mut App, key: K) {
        self.selector.select(cx, key);
    }

    /// Clear the current selection.
    pub fn clear(&self, cx: &mut App) {
        self.selector.clear(cx);
    }

    /// Set the selected key.
    pub fn set(&self, cx: &mut App, selected: Option<K>) {
        self.selector.set(cx, selected);
    }

    /// Select the next key from an ordered key set.
    pub fn select_next(&self, cx: &mut App, keys: impl IntoIterator<Item = K>) -> bool {
        self.selector.select_next(cx, keys)
    }

    /// Select the previous key from an ordered key set.
    pub fn select_previous(&self, cx: &mut App, keys: impl IntoIterator<Item = K>) -> bool {
        self.selector.select_previous(cx, keys)
    }

    /// Select the first key from an ordered key set.
    pub fn select_first(&self, cx: &mut App, keys: impl IntoIterator<Item = K>) -> bool {
        self.selector.select_first(cx, keys)
    }

    /// Select the last key from an ordered key set.
    pub fn select_last(&self, cx: &mut App, keys: impl IntoIterator<Item = K>) -> bool {
        self.selector.select_last(cx, keys)
    }

    /// Reconcile this selection against the currently available keys.
    pub fn reconcile_keys(&self, cx: &mut App, keys: impl IntoIterator<Item = K>) -> bool {
        self.selector.reconcile_keys(cx, keys)
    }

    /// Reconcile this selection and select the first key when missing.
    pub fn reconcile_or_select_first(
        &self,
        cx: &mut App,
        keys: impl IntoIterator<Item = K>,
    ) -> bool {
        self.selector.reconcile_or_select_first(cx, keys)
    }

    /// Project the currently selected item from a signal-backed collection.
    pub fn selected_from_signal<T>(
        &self,
        cx: &mut App,
        items: &Signal<Vec<T>>,
        key: impl Fn(&T) -> K + 'static,
    ) -> Memo<Option<T>>
    where
        T: Clone + PartialEq + 'static,
    {
        items.selected_by(cx, self.selector.clone(), key)
    }

    /// Project the selected item, falling back to the first item when missing.
    pub fn selected_from_signal_or_first<T>(
        &self,
        cx: &mut App,
        items: &Signal<Vec<T>>,
        key: impl Fn(&T) -> K + 'static,
    ) -> Memo<Option<T>>
    where
        T: Clone + PartialEq + 'static,
    {
        items.selected_by_or_first(cx, self.selector.clone(), key)
    }

    /// Project the currently selected item from a memo-backed collection.
    pub fn selected_from_memo<T>(
        &self,
        cx: &mut App,
        items: &Memo<Vec<T>>,
        key: impl Fn(&T) -> K + 'static,
    ) -> Memo<Option<T>>
    where
        T: Clone + PartialEq + 'static,
    {
        items.selected_by(cx, self.selector.clone(), key)
    }

    /// Project the selected item from a memo-backed collection, falling back to the first item.
    pub fn selected_from_memo_or_first<T>(
        &self,
        cx: &mut App,
        items: &Memo<Vec<T>>,
        key: impl Fn(&T) -> K + 'static,
    ) -> Memo<Option<T>>
    where
        T: Clone + PartialEq + 'static,
    {
        items.selected_by_or_first(cx, self.selector.clone(), key)
    }
}

impl<K> Clone for SelectionModel<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    fn clone(&self) -> Self {
        Self {
            selector: self.selector.clone(),
            has_selection: self.has_selection.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui::TestApp;

    use crate::{Signal, init};

    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Item {
        id: u64,
        label: &'static str,
    }

    #[test]
    fn selection_model_tracks_selection_presence() {
        let mut app = TestApp::new();
        let selection = app.update(|cx| {
            init(cx);
            use_selection_model::<u64>(cx, None)
        });

        app.read(|cx| {
            assert!(!selection.has_selection().get(cx));
            assert_eq!(selection.get(cx), None);
        });

        app.update(|cx| selection.select(cx, 2));

        app.read(|cx| {
            assert!(selection.has_selection().get(cx));
            assert_eq!(selection.get(cx), Some(2));
        });

        app.update(|cx| selection.clear(cx));

        app.read(|cx| {
            assert!(!selection.has_selection().get(cx));
            assert_eq!(selection.get(cx), None);
        });
    }

    #[test]
    fn selection_model_projects_selected_item() {
        let mut app = TestApp::new();
        let (selection, selected) = app.update(|cx| {
            init(cx);
            let items = Signal::new(
                cx,
                vec![
                    Item {
                        id: 1,
                        label: "one",
                    },
                    Item {
                        id: 2,
                        label: "two",
                    },
                ],
            );
            let selection = use_selection_model(cx, Some(1));
            let selected = selection.selected_from_signal(cx, &items, |item| item.id);
            (selection, selected)
        });

        app.read(|cx| {
            assert_eq!(selected.get(cx).map(|item| item.label), Some("one"));
        });

        app.update(|cx| selection.select(cx, 2));

        app.read(|cx| {
            assert_eq!(selected.get(cx).map(|item| item.label), Some("two"));
        });
    }
}
