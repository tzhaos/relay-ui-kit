use std::{cell::RefCell, hash::Hash, rc::Rc};

use gpui::{App, Context};

use crate::{Effect, Memo, ReactiveContextExt, SelectedItemExt, Selector, Signal, batch};

/// A higher-level single-selection model built on top of [`Selector`].
pub struct SelectionModel<K> {
    selector: Selector<K>,
    has_selection: Memo<bool>,
}

/// Policy for reconciling a selection against the latest ordered keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionReconcilePolicy {
    /// Clear the selection when the selected key is no longer available.
    Clear,
    /// Select the first available key when the selected key is missing.
    SelectFirst,
}

/// A source-driven single-selection model with an owned ordered key snapshot.
///
/// This is the migration-oriented selection composable for list- and
/// picker-like surfaces. It packages:
///
/// - a [`SelectionModel`] for the selected key itself;
/// - an entity-scoped watch that keeps the latest visible/ordered keys;
/// - automatic reconcile when those keys change;
/// - latest-order navigation helpers such as `select_next()` with no caller-
///   side key plumbing.
pub struct OrderedSelectionModel<K> {
    selection: SelectionModel<K>,
    ordered_keys: Signal<Vec<K>>,
    policy: SelectionReconcilePolicy,
    _effect: Rc<Effect>,
}

/// Create a selection model with the given initial key.
pub fn use_selection_model<K>(cx: &mut App, selected: Option<K>) -> SelectionModel<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    SelectionModel::new(cx, selected)
}

/// Create a source-driven ordered selection model.
///
/// The `ordered_keys` closure declares the tracked list order for this GPUI
/// entity. Relay snapshots that order, stores it in the model, and reconciles
/// the selected key whenever the source changes according to `policy`.
pub fn use_ordered_selection_model<K, Owner, KeysFn>(
    cx: &mut Context<Owner>,
    selected: Option<K>,
    ordered_keys: KeysFn,
    policy: SelectionReconcilePolicy,
) -> OrderedSelectionModel<K>
where
    Owner: 'static,
    K: Clone + Eq + Hash + PartialEq + 'static,
    KeysFn: Fn(&App) -> Vec<K> + 'static,
{
    OrderedSelectionModel::new(cx, selected, ordered_keys, policy)
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

impl<K> OrderedSelectionModel<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    /// Create a source-driven ordered selection model.
    pub fn new<Owner, KeysFn>(
        cx: &mut Context<Owner>,
        selected: Option<K>,
        ordered_keys: KeysFn,
        policy: SelectionReconcilePolicy,
    ) -> Self
    where
        Owner: 'static,
        KeysFn: Fn(&App) -> Vec<K> + 'static,
    {
        let selection = SelectionModel::new(cx, selected);
        let ordered_keys_signal = Signal::new(cx, Vec::new());
        let latest_keys: Rc<RefCell<Option<Vec<K>>>> = Rc::new(RefCell::new(None));
        let latest_keys_for_sources = latest_keys.clone();
        let selection_for_effect = selection.clone();
        let ordered_keys_for_effect = ordered_keys_signal.clone();

        let effect = cx.watch(
            move |cx| {
                *latest_keys_for_sources.borrow_mut() = Some(ordered_keys(cx));
            },
            move |cx| {
                let Some(keys) = latest_keys.borrow_mut().take() else {
                    return;
                };

                batch(cx, |cx| {
                    ordered_keys_for_effect.set(cx, keys.clone());
                    reconcile_selection(&selection_for_effect, cx, keys, policy);
                });
            },
        );

        Self {
            selection,
            ordered_keys: ordered_keys_signal,
            policy,
            _effect: Rc::new(effect),
        }
    }

    /// Return the underlying single-selection model.
    pub fn selection(&self) -> &SelectionModel<K> {
        &self.selection
    }

    /// Return the current reconcile policy.
    pub fn policy(&self) -> SelectionReconcilePolicy {
        self.policy
    }

    /// Return the signal holding the latest ordered keys.
    pub fn ordered_keys_signal(&self) -> &Signal<Vec<K>> {
        &self.ordered_keys
    }

    /// Read the latest ordered keys with dependency tracking.
    pub fn read_ordered_keys<R>(&self, cx: &App, f: impl FnOnce(&[K]) -> R) -> R {
        self.ordered_keys.read(cx, |keys| f(keys.as_slice()))
    }

    /// Clone the latest ordered keys with dependency tracking.
    pub fn ordered_keys(&self, cx: &App) -> Vec<K> {
        self.ordered_keys.get(cx)
    }

    /// Clone the latest ordered keys without dependency tracking.
    pub fn ordered_keys_untracked(&self) -> Vec<K> {
        self.ordered_keys.get_untracked()
    }

    /// Return a memo that is `true` when any key is selected.
    pub fn has_selection(&self) -> &Memo<bool> {
        self.selection.has_selection()
    }

    /// Clone the selected key with dependency tracking.
    pub fn get(&self, cx: &App) -> Option<K> {
        self.selection.get(cx)
    }

    /// Clone the selected key without dependency tracking.
    pub fn get_untracked(&self) -> Option<K> {
        self.selection.get_untracked()
    }

    /// Return whether the given key is selected.
    pub fn is_selected(&self, cx: &mut App, key: K) -> bool {
        self.selection.is_selected(cx, key)
    }

    /// Select a key.
    pub fn select(&self, cx: &mut App, key: K) {
        self.selection.select(cx, key);
    }

    /// Clear the current selection.
    pub fn clear(&self, cx: &mut App) {
        self.selection.clear(cx);
    }

    /// Set the selected key.
    pub fn set(&self, cx: &mut App, selected: Option<K>) {
        self.selection.set(cx, selected);
    }

    /// Reconcile the current selection against the latest ordered keys snapshot.
    pub fn reconcile_now(&self, cx: &mut App) -> bool {
        self.ordered_keys.peek(|keys| {
            reconcile_selection(&self.selection, cx, keys.iter().cloned(), self.policy)
        })
    }

    /// Select the next key from the latest ordered keys snapshot.
    pub fn select_next(&self, cx: &mut App) -> bool {
        self.ordered_keys
            .peek(|keys| self.selection.select_next(cx, keys.iter().cloned()))
    }

    /// Select the previous key from the latest ordered keys snapshot.
    pub fn select_previous(&self, cx: &mut App) -> bool {
        self.ordered_keys
            .peek(|keys| self.selection.select_previous(cx, keys.iter().cloned()))
    }

    /// Select the first key from the latest ordered keys snapshot.
    pub fn select_first(&self, cx: &mut App) -> bool {
        self.ordered_keys
            .peek(|keys| self.selection.select_first(cx, keys.iter().cloned()))
    }

    /// Select the last key from the latest ordered keys snapshot.
    pub fn select_last(&self, cx: &mut App) -> bool {
        self.ordered_keys
            .peek(|keys| self.selection.select_last(cx, keys.iter().cloned()))
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
        self.selection.selected_from_signal(cx, items, key)
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
        self.selection.selected_from_signal_or_first(cx, items, key)
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
        self.selection.selected_from_memo(cx, items, key)
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
        self.selection.selected_from_memo_or_first(cx, items, key)
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

impl<K> Clone for OrderedSelectionModel<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    fn clone(&self) -> Self {
        Self {
            selection: self.selection.clone(),
            ordered_keys: self.ordered_keys.clone(),
            policy: self.policy,
            _effect: self._effect.clone(),
        }
    }
}

fn reconcile_selection<K>(
    selection: &SelectionModel<K>,
    cx: &mut App,
    keys: impl IntoIterator<Item = K>,
    policy: SelectionReconcilePolicy,
) -> bool
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    match policy {
        SelectionReconcilePolicy::Clear => selection.reconcile_keys(cx, keys),
        SelectionReconcilePolicy::SelectFirst => selection.reconcile_or_select_first(cx, keys),
    }
}

#[cfg(test)]
mod tests {
    use gpui::{AppContext, Context, TestApp, TestAppContext};

    use crate::{Signal, init};

    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Item {
        id: u64,
        label: &'static str,
    }

    impl Item {
        fn new(id: u64, label: &'static str) -> Self {
            Self { id, label }
        }
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
            let items = Signal::new(cx, vec![Item::new(1, "one"), Item::new(2, "two")]);
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

    #[gpui::test]
    fn ordered_selection_model_selects_first_key_on_initial_run_when_configured(
        cx: &mut TestAppContext,
    ) {
        struct Host {
            keys: Signal<Vec<u64>>,
            selection: OrderedSelectionModel<u64>,
        }

        impl Host {
            fn new(cx: &mut Context<Self>) -> Self {
                init(cx);
                let keys = Signal::new(cx, vec![2, 4, 6]);
                let keys_for_selection = keys.clone();
                let selection = use_ordered_selection_model(
                    cx,
                    None,
                    move |cx| keys_for_selection.get(cx),
                    SelectionReconcilePolicy::SelectFirst,
                );

                Self { keys, selection }
            }
        }

        let entity = cx.update(|cx| cx.new(Host::new));

        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.keys.get(cx), vec![2, 4, 6]);
            assert_eq!(host.selection.ordered_keys(cx), vec![2, 4, 6]);
            assert_eq!(host.selection.get(cx), Some(2));
        });
    }

    #[gpui::test]
    fn ordered_selection_model_clears_missing_selection_when_policy_is_clear(
        cx: &mut TestAppContext,
    ) {
        struct Host {
            keys: Signal<Vec<u64>>,
            selection: OrderedSelectionModel<u64>,
        }

        impl Host {
            fn new(cx: &mut Context<Self>) -> Self {
                init(cx);
                let keys = Signal::new(cx, vec![1, 2, 3]);
                let keys_for_selection = keys.clone();
                let selection = use_ordered_selection_model(
                    cx,
                    Some(3),
                    move |cx| keys_for_selection.get(cx),
                    SelectionReconcilePolicy::Clear,
                );

                Self { keys, selection }
            }
        }

        let entity = cx.update(|cx| cx.new(Host::new));

        cx.update_entity(&entity, |host, cx| {
            host.keys.set(cx, vec![1, 2]);
        });

        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.selection.ordered_keys(cx), vec![1, 2]);
            assert_eq!(host.selection.get(cx), None);
        });
    }

    #[gpui::test]
    fn ordered_selection_model_navigation_uses_latest_filtered_key_order(cx: &mut TestAppContext) {
        struct Host {
            filter: Signal<&'static str>,
            items: Signal<Vec<Item>>,
            selection: OrderedSelectionModel<u64>,
        }

        impl Host {
            fn new(cx: &mut Context<Self>) -> Self {
                init(cx);
                let filter = Signal::new(cx, "al");
                let items = Signal::new(
                    cx,
                    vec![
                        Item::new(1, "alpha"),
                        Item::new(2, "beta"),
                        Item::new(3, "alpine"),
                    ],
                );
                let filter_for_selection = filter.clone();
                let items_for_selection = items.clone();
                let selection = use_ordered_selection_model(
                    cx,
                    Some(3),
                    move |cx| {
                        let filter = filter_for_selection.get(cx);
                        items_for_selection.read(cx, |items| {
                            items
                                .iter()
                                .filter(|item| item.label.contains(filter))
                                .map(|item| item.id)
                                .collect()
                        })
                    },
                    SelectionReconcilePolicy::SelectFirst,
                );

                Self {
                    filter,
                    items,
                    selection,
                }
            }
        }

        let entity = cx.update(|cx| cx.new(Host::new));

        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.items.get(cx).len(), 3);
            assert_eq!(host.selection.ordered_keys(cx), vec![1, 3]);
            assert_eq!(host.selection.get(cx), Some(3));
        });

        cx.update_entity(&entity, |host, cx| {
            assert!(host.selection.select_next(cx));
        });

        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.selection.get(cx), Some(1));
        });

        cx.update_entity(&entity, |host, cx| {
            host.filter.set(cx, "be");
        });

        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.selection.ordered_keys(cx), vec![2]);
            assert_eq!(host.selection.get(cx), Some(2));
        });

        cx.update_entity(&entity, |host, cx| {
            assert!(!host.selection.select_previous(cx));
        });

        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.selection.get(cx), Some(2));
        });
    }
}
