use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    hash::Hash,
    rc::Rc,
};

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

/// A source-driven multi-selection model with an active key and ordered marks.
///
/// This packages the workbench-style selection shape used by tree, panel, and
/// picker surfaces:
///
/// - an active/focused key;
/// - a marked selection set;
/// - per-key membership tracking;
/// - automatic reconcile against the latest ordered key snapshot;
/// - range and select-all helpers built on that latest order.
pub struct MultiSelectionModel<K> {
    active: SelectionModel<K>,
    selected_keys: Signal<Vec<K>>,
    ordered_keys: Signal<Vec<K>>,
    anchor: Signal<Option<K>>,
    has_selection: Memo<bool>,
    selection_count: Memo<usize>,
    keyed: Rc<RefCell<HashMap<K, Signal<bool>>>>,
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

/// Create a source-driven multi-selection model.
///
/// Relay keeps both the active key and the marked selection set aligned with
/// the latest ordered keys. This matches GPUI surfaces that need an active row
/// plus a broader marked set for batch actions, drag-and-drop, or range
/// selection.
pub fn use_multi_selection_model<K, Owner, KeysFn, Selected>(
    cx: &mut Context<Owner>,
    active: Option<K>,
    selected: Selected,
    ordered_keys: KeysFn,
    policy: SelectionReconcilePolicy,
) -> MultiSelectionModel<K>
where
    Owner: 'static,
    K: Clone + Eq + Hash + PartialEq + 'static,
    KeysFn: Fn(&App) -> Vec<K> + 'static,
    Selected: IntoIterator<Item = K>,
{
    MultiSelectionModel::new(cx, active, selected, ordered_keys, policy)
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

impl<K> MultiSelectionModel<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    /// Create a source-driven multi-selection model.
    pub fn new<Owner, KeysFn, Selected>(
        cx: &mut Context<Owner>,
        active: Option<K>,
        selected: Selected,
        ordered_keys: KeysFn,
        policy: SelectionReconcilePolicy,
    ) -> Self
    where
        Owner: 'static,
        KeysFn: Fn(&App) -> Vec<K> + 'static,
        Selected: IntoIterator<Item = K>,
    {
        let mut initial_selected = dedupe_keys(selected);
        let initial_active = active.or_else(|| initial_selected.first().cloned());
        if let Some(active) = initial_active.clone()
            && !initial_selected.iter().any(|selected| selected == &active)
        {
            initial_selected.push(active);
        }

        let active_selection = SelectionModel::new(cx, initial_active.clone());
        let selected_keys = Signal::new(cx, initial_selected);
        let ordered_keys_signal = Signal::new(cx, Vec::new());
        let anchor = Signal::new(cx, initial_active.clone());
        let keyed: Rc<RefCell<HashMap<K, Signal<bool>>>> = Rc::default();

        let selected_keys_for_has_selection = selected_keys.clone();
        let selected_keys_for_count = selected_keys.clone();
        let latest_keys: Rc<RefCell<Option<Vec<K>>>> = Rc::new(RefCell::new(None));
        let latest_keys_for_sources = latest_keys.clone();
        let active_for_effect = active_selection.clone();
        let selected_keys_for_effect = selected_keys.clone();
        let ordered_keys_for_effect = ordered_keys_signal.clone();
        let anchor_for_effect = anchor.clone();
        let keyed_for_effect = keyed.clone();

        let effect = cx.watch(
            move |cx| {
                *latest_keys_for_sources.borrow_mut() = Some(dedupe_keys(ordered_keys(cx)));
            },
            move |cx| {
                let Some(keys) = latest_keys.borrow_mut().take() else {
                    return;
                };

                batch(cx, |cx| {
                    ordered_keys_for_effect.set(cx, keys.clone());
                    reconcile_multi_selection(
                        &active_for_effect,
                        &selected_keys_for_effect,
                        &anchor_for_effect,
                        &keyed_for_effect,
                        cx,
                        keys.as_slice(),
                        policy,
                    );
                });
            },
        );

        Self {
            active: active_selection,
            selected_keys,
            ordered_keys: ordered_keys_signal,
            anchor,
            has_selection: Memo::new(cx, move |cx| {
                !selected_keys_for_has_selection.get(cx).is_empty()
            }),
            selection_count: Memo::new(cx, move |cx| {
                selected_keys_for_count.read(cx, |selected| selected.len())
            }),
            keyed,
            policy,
            _effect: Rc::new(effect),
        }
    }

    /// Return the current reconcile policy.
    pub fn policy(&self) -> SelectionReconcilePolicy {
        self.policy
    }

    /// Return a memo that is `true` when any key is selected.
    pub fn has_selection(&self) -> &Memo<bool> {
        &self.has_selection
    }

    /// Return a memo with the current selected key count.
    pub fn selection_count(&self) -> &Memo<usize> {
        &self.selection_count
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

    /// Return the signal holding the current marked selection keys.
    pub fn selected_keys_signal(&self) -> &Signal<Vec<K>> {
        &self.selected_keys
    }

    /// Read the current marked selection keys with dependency tracking.
    pub fn read_selected_keys<R>(&self, cx: &App, f: impl FnOnce(&[K]) -> R) -> R {
        self.selected_keys.read(cx, |keys| f(keys.as_slice()))
    }

    /// Clone the current marked selection keys with dependency tracking.
    pub fn selected_keys(&self, cx: &App) -> Vec<K> {
        self.selected_keys.get(cx)
    }

    /// Clone the current marked selection keys without dependency tracking.
    pub fn selected_keys_untracked(&self) -> Vec<K> {
        self.selected_keys.get_untracked()
    }

    /// Clone the active key with dependency tracking.
    pub fn active(&self, cx: &App) -> Option<K> {
        self.active.get(cx)
    }

    /// Clone the active key without dependency tracking.
    pub fn active_untracked(&self) -> Option<K> {
        self.active.get_untracked()
    }

    /// Return whether the given key is the active key.
    pub fn is_active(&self, cx: &mut App, key: K) -> bool {
        self.active.is_selected(cx, key)
    }

    /// Return whether the given key is part of the marked selection set.
    pub fn is_selected(&self, cx: &mut App, key: K) -> bool {
        self.key_signal(cx, key).get(cx)
    }

    /// Replace the current selection with a single active key.
    pub fn select_only(&self, cx: &mut App, key: K) -> bool {
        self.apply_state(cx, vec![key.clone()], Some(key.clone()), Some(key))
    }

    /// Add a key to the marked selection set and make it active.
    pub fn add(&self, cx: &mut App, key: K) -> bool {
        let mut selected = self.selected_keys.get_untracked();
        selected.push(key.clone());
        let selected = self.normalize_selected_keys(selected);
        self.apply_state(cx, selected, Some(key.clone()), Some(key))
    }

    /// Remove a key from the marked selection set.
    pub fn remove(&self, cx: &mut App, key: K) -> bool {
        let previous_selected = self.selected_keys.get_untracked();
        if !previous_selected.iter().any(|selected| selected == &key) {
            return false;
        }

        let selected = self.normalize_selected_keys(
            previous_selected
                .into_iter()
                .filter(|selected| selected != &key),
        );
        let active = match self.active.get_untracked() {
            Some(active) if active == key => selected.first().cloned(),
            Some(active) if selected.iter().any(|selected| selected == &active) => Some(active),
            _ => selected.first().cloned(),
        };
        let anchor = self.anchor.get_untracked().filter(|anchor| anchor != &key);

        self.apply_state(cx, selected, active, anchor)
    }

    /// Toggle a key in the marked selection set.
    pub fn toggle(&self, cx: &mut App, key: K) -> bool {
        if self
            .selected_keys
            .peek(|selected| selected.iter().any(|selected_key| selected_key == &key))
        {
            self.remove(cx, key)
        } else {
            self.add(cx, key)
        }
    }

    /// Clear the marked selection set and active key.
    pub fn clear(&self, cx: &mut App) -> bool {
        self.apply_state(cx, Vec::new(), None, None)
    }

    /// Select every key in the latest ordered snapshot.
    pub fn select_all(&self, cx: &mut App) -> bool {
        let selected = self.ordered_keys_untracked();
        let active = self
            .active
            .get_untracked()
            .filter(|active| selected.iter().any(|selected| selected == active))
            .or_else(|| selected.first().cloned());

        self.apply_state(cx, selected, active.clone(), active)
    }

    /// Replace the marked selection with the range from the anchor to `key`.
    pub fn extend_to(&self, cx: &mut App, key: K) -> bool {
        let ordered_keys = self.ordered_keys_untracked();
        if ordered_keys.is_empty() {
            return self.select_only(cx, key);
        }

        let Some(target_index) = ordered_keys.iter().position(|ordered| ordered == &key) else {
            return self.select_only(cx, key);
        };
        let anchor = self
            .anchor
            .get_untracked()
            .filter(|anchor| ordered_keys.iter().any(|ordered| ordered == anchor))
            .or_else(|| {
                self.active
                    .get_untracked()
                    .filter(|active| ordered_keys.iter().any(|ordered| ordered == active))
            })
            .unwrap_or_else(|| key.clone());
        let Some(anchor_index) = ordered_keys.iter().position(|ordered| ordered == &anchor) else {
            return self.select_only(cx, key);
        };
        let (start, end) = if anchor_index <= target_index {
            (anchor_index, target_index)
        } else {
            (target_index, anchor_index)
        };
        let selected = ordered_keys[start..=end].to_vec();

        self.apply_state(cx, selected, Some(key), Some(anchor))
    }

    /// Collapse to the next ordered key.
    pub fn select_next_only(&self, cx: &mut App) -> bool {
        self.select_relative_only(cx, SelectionStep::Next)
    }

    /// Collapse to the previous ordered key.
    pub fn select_previous_only(&self, cx: &mut App) -> bool {
        self.select_relative_only(cx, SelectionStep::Previous)
    }

    /// Collapse to the first ordered key.
    pub fn select_first_only(&self, cx: &mut App) -> bool {
        self.select_boundary_only(cx, SelectionBoundary::First)
    }

    /// Collapse to the last ordered key.
    pub fn select_last_only(&self, cx: &mut App) -> bool {
        self.select_boundary_only(cx, SelectionBoundary::Last)
    }

    /// Extend the current range to the next ordered key.
    pub fn extend_next(&self, cx: &mut App) -> bool {
        self.extend_relative(cx, SelectionStep::Next)
    }

    /// Extend the current range to the previous ordered key.
    pub fn extend_previous(&self, cx: &mut App) -> bool {
        self.extend_relative(cx, SelectionStep::Previous)
    }

    /// Reconcile the current marked selection against the latest ordered keys snapshot.
    pub fn reconcile_now(&self, cx: &mut App) -> bool {
        self.ordered_keys.peek(|keys| {
            reconcile_multi_selection(
                &self.active,
                &self.selected_keys,
                &self.anchor,
                &self.keyed,
                cx,
                keys.as_slice(),
                self.policy,
            )
        })
    }

    /// Project the active item from a signal-backed collection.
    pub fn active_from_signal<T>(
        &self,
        cx: &mut App,
        items: &Signal<Vec<T>>,
        key: impl Fn(&T) -> K + 'static,
    ) -> Memo<Option<T>>
    where
        T: Clone + PartialEq + 'static,
    {
        self.active.selected_from_signal(cx, items, key)
    }

    /// Project the active item from a memo-backed collection.
    pub fn active_from_memo<T>(
        &self,
        cx: &mut App,
        items: &Memo<Vec<T>>,
        key: impl Fn(&T) -> K + 'static,
    ) -> Memo<Option<T>>
    where
        T: Clone + PartialEq + 'static,
    {
        self.active.selected_from_memo(cx, items, key)
    }

    /// Project the marked items from a signal-backed collection in selection order.
    pub fn selected_items_from_signal<T>(
        &self,
        cx: &mut App,
        items: &Signal<Vec<T>>,
        key: impl Fn(&T) -> K + 'static,
    ) -> Memo<Vec<T>>
    where
        T: Clone + PartialEq + 'static,
    {
        let items = items.clone();
        let selected_keys = self.selected_keys.clone();

        Memo::new(cx, move |cx| {
            let selected_keys = selected_keys.get(cx);
            items.read(cx, |items| {
                selected_items(items, selected_keys.as_slice(), &key)
            })
        })
    }

    /// Project the marked items from a memo-backed collection in selection order.
    pub fn selected_items_from_memo<T>(
        &self,
        cx: &mut App,
        items: &Memo<Vec<T>>,
        key: impl Fn(&T) -> K + 'static,
    ) -> Memo<Vec<T>>
    where
        T: Clone + PartialEq + 'static,
    {
        let items = items.clone();
        let selected_keys = self.selected_keys.clone();

        Memo::new(cx, move |cx| {
            let selected_keys = selected_keys.get(cx);
            items.read(cx, |items| {
                selected_items(items, selected_keys.as_slice(), &key)
            })
        })
    }

    fn apply_state(
        &self,
        cx: &mut App,
        mut selected: Vec<K>,
        active: Option<K>,
        anchor: Option<K>,
    ) -> bool {
        selected = dedupe_keys(selected);
        if let Some(active) = active.as_ref()
            && !selected.iter().any(|selected_key| selected_key == active)
        {
            selected.push(active.clone());
        }

        let active = if selected.is_empty() {
            None
        } else {
            active
                .filter(|active| selected.iter().any(|selected_key| selected_key == active))
                .or_else(|| selected.first().cloned())
        };
        let anchor = anchor
            .filter(|anchor| selected.iter().any(|selected_key| selected_key == anchor))
            .or_else(|| active.clone());

        apply_multi_selection_state(
            &self.active,
            &self.selected_keys,
            &self.anchor,
            &self.keyed,
            cx,
            selected,
            active,
            anchor,
        )
    }

    fn key_signal(&self, cx: &mut App, key: K) -> Signal<bool> {
        if let Some(signal) = self.keyed.borrow().get(&key) {
            return signal.clone();
        }

        let is_selected = self
            .selected_keys
            .peek(|selected| selected.iter().any(|selected_key| selected_key == &key));
        let signal = Signal::new(cx, is_selected);
        self.keyed.borrow_mut().insert(key, signal.clone());
        signal
    }

    fn normalize_selected_keys(&self, keys: impl IntoIterator<Item = K>) -> Vec<K> {
        self.ordered_keys
            .peek(|ordered_keys| normalize_keys_to_order(keys, ordered_keys.as_slice()))
    }

    fn select_relative_only(&self, cx: &mut App, step: SelectionStep) -> bool {
        let ordered_keys = self.ordered_keys_untracked();
        match relative_key(&ordered_keys, self.active.get_untracked().as_ref(), step) {
            Some(key) => self.select_only(cx, key),
            None => self.clear(cx),
        }
    }

    fn select_boundary_only(&self, cx: &mut App, boundary: SelectionBoundary) -> bool {
        let ordered_keys = self.ordered_keys_untracked();
        match boundary_key(&ordered_keys, boundary) {
            Some(key) => self.select_only(cx, key),
            None => self.clear(cx),
        }
    }

    fn extend_relative(&self, cx: &mut App, step: SelectionStep) -> bool {
        let ordered_keys = self.ordered_keys_untracked();
        match relative_key(&ordered_keys, self.active.get_untracked().as_ref(), step) {
            Some(key) => self.extend_to(cx, key),
            None => self.clear(cx),
        }
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

impl<K> Clone for MultiSelectionModel<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    fn clone(&self) -> Self {
        Self {
            active: self.active.clone(),
            selected_keys: self.selected_keys.clone(),
            ordered_keys: self.ordered_keys.clone(),
            anchor: self.anchor.clone(),
            has_selection: self.has_selection.clone(),
            selection_count: self.selection_count.clone(),
            keyed: self.keyed.clone(),
            policy: self.policy,
            _effect: self._effect.clone(),
        }
    }
}

#[derive(Clone, Copy)]
enum SelectionStep {
    Next,
    Previous,
}

#[derive(Clone, Copy)]
enum SelectionBoundary {
    First,
    Last,
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

fn reconcile_multi_selection<K>(
    active: &SelectionModel<K>,
    selected_keys: &Signal<Vec<K>>,
    anchor: &Signal<Option<K>>,
    keyed: &Rc<RefCell<HashMap<K, Signal<bool>>>>,
    cx: &mut App,
    ordered_keys: &[K],
    policy: SelectionReconcilePolicy,
) -> bool
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    let previous_selected = selected_keys.get_untracked();
    let selected_set = previous_selected.iter().cloned().collect::<HashSet<_>>();
    let mut next_selected = ordered_keys
        .iter()
        .filter(|key| selected_set.contains(*key))
        .cloned()
        .collect::<Vec<_>>();

    let next_active = match active.get_untracked() {
        Some(active_key) if next_selected.iter().any(|selected| selected == &active_key) => {
            Some(active_key)
        }
        _ if !next_selected.is_empty() => next_selected.first().cloned(),
        _ => match policy {
            SelectionReconcilePolicy::Clear => None,
            SelectionReconcilePolicy::SelectFirst => ordered_keys.first().cloned(),
        },
    };

    if next_selected.is_empty()
        && let Some(active_key) = next_active.clone()
    {
        next_selected.push(active_key);
    }

    let next_anchor = anchor
        .get_untracked()
        .filter(|anchor_key| next_selected.iter().any(|selected| selected == anchor_key))
        .or_else(|| next_active.clone());
    let changed = apply_multi_selection_state(
        active,
        selected_keys,
        anchor,
        keyed,
        cx,
        next_selected,
        next_active,
        next_anchor,
    );
    retain_multi_selection_keyed_signals(keyed, ordered_keys.iter().cloned());
    changed
}

fn apply_multi_selection_state<K>(
    active: &SelectionModel<K>,
    selected_keys: &Signal<Vec<K>>,
    anchor: &Signal<Option<K>>,
    keyed: &Rc<RefCell<HashMap<K, Signal<bool>>>>,
    cx: &mut App,
    selected: Vec<K>,
    active_key: Option<K>,
    anchor_key: Option<K>,
) -> bool
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    let previous_selected = selected_keys.get_untracked();
    let previous_active = active.get_untracked();
    let previous_anchor = anchor.get_untracked();
    if previous_selected == selected
        && previous_active == active_key
        && previous_anchor == anchor_key
    {
        return false;
    }

    batch(cx, |cx| {
        if previous_selected != selected {
            selected_keys.set(cx, selected.clone());
            update_keyed_membership_signals(
                keyed,
                cx,
                previous_selected.as_slice(),
                selected.as_slice(),
            );
        }
        if previous_active != active_key {
            active.set(cx, active_key.clone());
        }
        if previous_anchor != anchor_key {
            anchor.set(cx, anchor_key);
        }
    });

    true
}

fn update_keyed_membership_signals<K>(
    keyed: &Rc<RefCell<HashMap<K, Signal<bool>>>>,
    cx: &mut App,
    previous_selected: &[K],
    next_selected: &[K],
) where
    K: Clone + Eq + Hash,
{
    let previous_selected = previous_selected.iter().cloned().collect::<HashSet<_>>();
    let next_selected = next_selected.iter().cloned().collect::<HashSet<_>>();
    let keyed = keyed.borrow();

    for removed in previous_selected.difference(&next_selected) {
        if let Some(signal) = keyed.get(removed) {
            signal.set(cx, false);
        }
    }

    for added in next_selected.difference(&previous_selected) {
        if let Some(signal) = keyed.get(added) {
            signal.set(cx, true);
        }
    }
}

fn retain_multi_selection_keyed_signals<K>(
    keyed: &Rc<RefCell<HashMap<K, Signal<bool>>>>,
    keys: impl IntoIterator<Item = K>,
) where
    K: Clone + Eq + Hash,
{
    let keys = keys.into_iter().collect::<HashSet<_>>();
    keyed.borrow_mut().retain(|key, _| keys.contains(key));
}

fn dedupe_keys<K>(keys: impl IntoIterator<Item = K>) -> Vec<K>
where
    K: Clone + Eq + Hash,
{
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();

    for key in keys {
        if seen.insert(key.clone()) {
            deduped.push(key);
        }
    }

    deduped
}

fn normalize_keys_to_order<K>(keys: impl IntoIterator<Item = K>, ordered_keys: &[K]) -> Vec<K>
where
    K: Clone + Eq + Hash,
{
    let input = dedupe_keys(keys);
    if input.is_empty() {
        return input;
    }
    if ordered_keys.is_empty() {
        return input;
    }

    let input_set = input.iter().cloned().collect::<HashSet<_>>();
    let mut normalized = ordered_keys
        .iter()
        .filter(|key| input_set.contains(*key))
        .cloned()
        .collect::<Vec<_>>();
    let mut normalized_set = normalized.iter().cloned().collect::<HashSet<_>>();

    for key in input {
        if normalized_set.insert(key.clone()) {
            normalized.push(key);
        }
    }

    normalized
}

fn relative_key<K>(ordered_keys: &[K], current: Option<&K>, step: SelectionStep) -> Option<K>
where
    K: Clone + PartialEq,
{
    if ordered_keys.is_empty() {
        return None;
    }

    let selected_index =
        current.and_then(|current| ordered_keys.iter().position(|key| key == current));
    let next_index = match (selected_index, step) {
        (Some(index), SelectionStep::Next) => (index + 1) % ordered_keys.len(),
        (Some(0), SelectionStep::Previous) => ordered_keys.len() - 1,
        (Some(index), SelectionStep::Previous) => index - 1,
        (None, SelectionStep::Next) => 0,
        (None, SelectionStep::Previous) => ordered_keys.len() - 1,
    };

    Some(ordered_keys[next_index].clone())
}

fn boundary_key<K>(ordered_keys: &[K], boundary: SelectionBoundary) -> Option<K>
where
    K: Clone,
{
    match boundary {
        SelectionBoundary::First => ordered_keys.first().cloned(),
        SelectionBoundary::Last => ordered_keys.last().cloned(),
    }
}

fn selected_items<T, K>(items: &[T], selected_keys: &[K], key: &impl Fn(&T) -> K) -> Vec<T>
where
    T: Clone,
    K: Clone + Eq + Hash,
{
    if selected_keys.is_empty() {
        return Vec::new();
    }

    let items_by_key = items
        .iter()
        .map(|item| (key(item), item.clone()))
        .collect::<HashMap<_, _>>();

    selected_keys
        .iter()
        .filter_map(|selected_key| items_by_key.get(selected_key).cloned())
        .collect()
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

    #[gpui::test]
    fn multi_selection_model_selects_first_key_on_initial_run_when_configured(
        cx: &mut TestAppContext,
    ) {
        struct Host {
            keys: Signal<Vec<u64>>,
            selection: MultiSelectionModel<u64>,
        }

        impl Host {
            fn new(cx: &mut Context<Self>) -> Self {
                init(cx);
                let keys = Signal::new(cx, vec![2, 4, 6]);
                let keys_for_selection = keys.clone();
                let selection = use_multi_selection_model(
                    cx,
                    None,
                    Vec::<u64>::new(),
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
            assert_eq!(host.selection.selected_keys(cx), vec![2]);
            assert_eq!(host.selection.active(cx), Some(2));
            assert!(host.selection.has_selection().get(cx));
            assert_eq!(host.selection.selection_count().get(cx), 1);
        });
    }

    #[gpui::test]
    fn multi_selection_model_range_and_reconcile_use_latest_filtered_order(
        cx: &mut TestAppContext,
    ) {
        #[derive(Clone, Debug, PartialEq, Eq)]
        struct Entry {
            id: u64,
            group: &'static str,
            label: &'static str,
        }

        impl Entry {
            fn new(id: u64, group: &'static str, label: &'static str) -> Self {
                Self { id, group, label }
            }
        }

        struct Host {
            filter: Signal<&'static str>,
            items: Signal<Vec<Entry>>,
            selection: MultiSelectionModel<u64>,
            selected_items: Memo<Vec<Entry>>,
            active_item: Memo<Option<Entry>>,
        }

        impl Host {
            fn new(cx: &mut Context<Self>) -> Self {
                init(cx);
                let filter = Signal::new(cx, "src");
                let items = Signal::new(
                    cx,
                    vec![
                        Entry::new(1, "src", "alpha"),
                        Entry::new(2, "docs", "beta"),
                        Entry::new(3, "src", "alpine"),
                        Entry::new(4, "docs", "branch"),
                    ],
                );
                let filter_for_selection = filter.clone();
                let items_for_selection = items.clone();
                let selection = use_multi_selection_model(
                    cx,
                    Some(1),
                    [1],
                    move |cx| {
                        let filter = filter_for_selection.get(cx);
                        items_for_selection.read(cx, |items| {
                            items
                                .iter()
                                .filter(|item| item.group == filter)
                                .map(|item| item.id)
                                .collect()
                        })
                    },
                    SelectionReconcilePolicy::SelectFirst,
                );
                let selected_items =
                    selection.selected_items_from_signal(cx, &items, |item| item.id);
                let active_item = selection.active_from_signal(cx, &items, |item| item.id);

                Self {
                    filter,
                    items,
                    selection,
                    selected_items,
                    active_item,
                }
            }
        }

        let entity = cx.update(|cx| cx.new(Host::new));

        cx.update_entity(&entity, |host, cx| {
            assert!(host.selection.extend_to(cx, 3));
        });

        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.items.get(cx).len(), 4);
            assert_eq!(host.selection.selected_keys(cx), vec![1, 3]);
            assert_eq!(host.selection.active(cx), Some(3));
            assert_eq!(
                host.active_item.get(cx).map(|item| item.label),
                Some("alpine")
            );
            assert_eq!(
                host.selected_items.read(cx, |items| items
                    .iter()
                    .map(|item| item.label)
                    .collect::<Vec<_>>()),
                vec!["alpha", "alpine"]
            );
        });

        cx.update_entity(&entity, |host, cx| {
            host.filter.set(cx, "docs");
        });

        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.selection.ordered_keys(cx), vec![2, 4]);
            assert_eq!(host.selection.selected_keys(cx), vec![2]);
            assert_eq!(host.selection.active(cx), Some(2));
            assert_eq!(
                host.selected_items.read(cx, |items| items
                    .iter()
                    .map(|item| item.label)
                    .collect::<Vec<_>>()),
                vec!["beta"]
            );
        });
    }

    #[gpui::test]
    fn multi_selection_model_remove_active_rehomes_to_first_remaining_selected_key(
        cx: &mut TestAppContext,
    ) {
        struct Host {
            selection: MultiSelectionModel<u64>,
        }

        impl Host {
            fn new(cx: &mut Context<Self>) -> Self {
                init(cx);
                let keys = Signal::new(cx, vec![1, 2, 3]);
                let keys_for_selection = keys.clone();
                let selection = use_multi_selection_model(
                    cx,
                    Some(2),
                    [1, 2, 3],
                    move |cx| keys_for_selection.get(cx),
                    SelectionReconcilePolicy::Clear,
                );

                Self { selection }
            }
        }

        let entity = cx.update(|cx| cx.new(Host::new));

        cx.update_entity(&entity, |host, cx| {
            assert!(host.selection.remove(cx, 2));
            assert!(!host.selection.is_selected(cx, 2));
        });

        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.selection.selected_keys(cx), vec![1, 3]);
            assert_eq!(host.selection.active(cx), Some(1));
            assert_eq!(host.selection.selection_count().get(cx), 2);
        });
    }

    #[gpui::test]
    fn multi_selection_model_select_all_uses_latest_order_and_count(cx: &mut TestAppContext) {
        struct Host {
            keys: Signal<Vec<u64>>,
            selection: MultiSelectionModel<u64>,
        }

        impl Host {
            fn new(cx: &mut Context<Self>) -> Self {
                init(cx);
                let keys = Signal::new(cx, vec![3, 1, 2]);
                let keys_for_selection = keys.clone();
                let selection = use_multi_selection_model(
                    cx,
                    None,
                    Vec::<u64>::new(),
                    move |cx| keys_for_selection.get(cx),
                    SelectionReconcilePolicy::Clear,
                );

                Self { keys, selection }
            }
        }

        let entity = cx.update(|cx| cx.new(Host::new));

        cx.update_entity(&entity, |host, cx| {
            assert!(host.selection.select_all(cx));
        });

        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.keys.get(cx), vec![3, 1, 2]);
            assert_eq!(host.selection.selected_keys(cx), vec![3, 1, 2]);
            assert_eq!(host.selection.active(cx), Some(3));
            assert!(host.selection.has_selection().get(cx));
            assert_eq!(host.selection.selection_count().get(cx), 3);
        });
    }
}
