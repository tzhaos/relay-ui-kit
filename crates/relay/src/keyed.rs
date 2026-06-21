//! Keyed GPUI entity helpers for list-shaped views.
//!
//! `KeyedSubViews` is the Relay primitive for list rows that deserve their own
//! GPUI entity boundary. It keeps row entities stable across insert, remove, and
//! reorder operations, then lets those rows render through GPUI's view cache.
//! Use ordinary element mapping for cheap stateless rows; use this module when
//! rows hold state, focus/scroll-like element state, subscriptions, resources,
//! or enough rendering work that clean siblings should stay cached.

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use gpui::{AnyElement, AppContext, Context, Render, StyleRefinement};

use crate::SubView;

/// One retained keyed row in a [`KeyedSubViews`] collection.
pub struct KeyedSubView<K, V: 'static> {
    key: K,
    view: SubView<V>,
}

impl<K, V: 'static> KeyedSubView<K, V> {
    /// Borrow this row's key.
    pub fn key(&self) -> &K {
        &self.key
    }

    /// Borrow this row's retained subview.
    pub fn view(&self) -> &SubView<V> {
        &self.view
    }
}

/// Retains one GPUI child entity per stable item key.
///
/// Use this when a list is made of real view entities, such as task rows,
/// file-tree rows, session rows, or log rows with state. `sync` reconciles the
/// collection against the latest item order, reusing existing row entities
/// when their keys are still present and dropping rows whose keys disappeared.
///
/// For lightweight rows that are just cheap GPUI elements, map the collection
/// directly in the component layer. `KeyedSubViews` pays for a child entity per
/// key because the entity is the cache and lifecycle boundary.
pub struct KeyedSubViews<K, V: 'static> {
    entries: Vec<KeyedSubView<K, V>>,
    indices: HashMap<K, usize>,
}

impl<K, V> KeyedSubViews<K, V>
where
    K: Clone + Eq + Hash + 'static,
    V: 'static,
{
    /// Create an empty keyed subview collection.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            indices: HashMap::new(),
        }
    }

    /// Return the number of retained rows.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return whether this collection contains no rows.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Drop all retained row entities.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.indices.clear();
    }

    /// Borrow the row for `key`, if it is currently retained.
    pub fn get(&self, key: &K) -> Option<&SubView<V>> {
        self.indices
            .get(key)
            .and_then(|index| self.entries.get(*index))
            .map(|entry| &entry.view)
    }

    /// Iterate retained rows in the current item order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &SubView<V>> {
        self.entries.iter().map(|entry| &entry.view)
    }

    /// Iterate retained keyed rows in the current item order.
    pub fn keyed_iter(&self) -> impl ExactSizeIterator<Item = (&K, &SubView<V>)> {
        self.entries.iter().map(|entry| (&entry.key, &entry.view))
    }

    /// Borrow retained keyed rows in the current item order.
    pub fn entries(&self) -> &[KeyedSubView<K, V>] {
        &self.entries
    }

    /// Reconcile the retained row entities against `items`.
    ///
    /// `key` must return a stable, unique key for every item in `items`.
    /// `build` creates a row entity for new keys. `update` refreshes an
    /// existing row entity for reused keys and returns whether that row should
    /// be notified.
    ///
    /// # Panics
    ///
    /// Panics when `items` contains duplicate keys.
    pub fn sync<T>(
        &mut self,
        cx: &mut impl AppContext,
        items: impl IntoIterator<Item = T>,
        mut key: impl FnMut(&T) -> K,
        mut build: impl FnMut(&T, &mut Context<V>) -> V,
        mut update: impl FnMut(&T, &mut V, &mut Context<V>) -> bool,
    ) {
        let mut previous = self
            .entries
            .drain(..)
            .map(|entry| (entry.key, entry.view))
            .collect::<HashMap<_, _>>();

        let mut next_entries = Vec::new();
        let mut next_indices = HashMap::new();
        let mut seen = HashSet::new();

        for item in items {
            let item_key = key(&item);
            assert!(
                seen.insert(item_key.clone()),
                "KeyedSubViews::sync received duplicate item keys"
            );

            let view = if let Some(view) = previous.remove(&item_key) {
                view.entity().update(cx, |row, cx| {
                    if update(&item, row, cx) {
                        cx.notify();
                    }
                });
                view
            } else {
                SubView::new(cx, |cx| build(&item, cx))
            };

            next_indices.insert(item_key.clone(), next_entries.len());
            next_entries.push(KeyedSubView {
                key: item_key,
                view,
            });
        }

        self.entries = next_entries;
        self.indices = next_indices;
    }
}

impl<K, V> KeyedSubViews<K, V>
where
    K: Clone + Eq + Hash + 'static,
    V: Render,
{
    /// Render all retained rows with GPUI's view cache.
    pub fn cached(&self, style: StyleRefinement) -> impl ExactSizeIterator<Item = AnyElement> + '_ {
        self.iter().map(move |view| view.cached(style.clone()))
    }

    /// Render all retained rows cached with a full-size root style.
    pub fn cached_full(&self) -> impl ExactSizeIterator<Item = AnyElement> + '_ {
        self.iter().map(|view| view.cached_full())
    }

    /// Render all retained rows without GPUI's view cache.
    pub fn uncached(&self) -> impl ExactSizeIterator<Item = AnyElement> + '_ {
        self.iter().map(|view| view.uncached())
    }
}

impl<K, V> Default for KeyedSubViews<K, V>
where
    K: Clone + Eq + Hash + 'static,
    V: 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{
        AnyElement, Context, EntityId, IntoElement, ParentElement, Render, Styled, TestApp, Window,
        div,
    };

    use crate::{ReactiveAppExt, ReactiveView, Signal, init, view::reactive_render};

    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Item {
        id: u64,
        label: String,
    }

    impl Item {
        fn new(id: u64, label: impl Into<String>) -> Self {
            Self {
                id,
                label: label.into(),
            }
        }
    }

    struct RowView {
        label: String,
        renders: Rc<Cell<usize>>,
    }

    impl RowView {
        fn new(item: &Item, renders: Rc<Cell<usize>>) -> Self {
            Self {
                label: item.label.clone(),
                renders,
            }
        }

        fn update_item(&mut self, item: &Item) -> bool {
            if self.label == item.label {
                false
            } else {
                self.label = item.label.clone();
                true
            }
        }
    }

    impl ReactiveView for RowView {
        fn render_state(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> AnyElement {
            self.renders.set(self.renders.get() + 1);
            div().child(self.label.clone()).into_any_element()
        }
    }

    impl Render for RowView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            reactive_render(self, window, cx)
        }
    }

    struct ListHost {
        items: Signal<Vec<Item>>,
        rows: KeyedSubViews<u64, RowView>,
        row_renders: Rc<Cell<usize>>,
        renders: Rc<Cell<usize>>,
    }

    impl ListHost {
        fn new(
            cx: &mut Context<Self>,
            row_renders: Rc<Cell<usize>>,
            renders: Rc<Cell<usize>>,
        ) -> Self {
            init(cx);
            Self {
                items: cx.signal(vec![
                    Item::new(1, "one"),
                    Item::new(2, "two"),
                    Item::new(3, "three"),
                ]),
                rows: KeyedSubViews::new(),
                row_renders,
                renders,
            }
        }
    }

    impl ReactiveView for ListHost {
        fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
            self.renders.set(self.renders.get() + 1);
            let items = self.items.get(cx);
            let row_renders = self.row_renders.clone();

            self.rows.sync(
                cx,
                items,
                |item| item.id,
                move |item, _cx| RowView::new(item, row_renders.clone()),
                |item, row, _cx| row.update_item(item),
            );

            div()
                .size_full()
                .children(self.rows.cached_full())
                .into_any_element()
        }
    }

    impl Render for ListHost {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            reactive_render(self, window, cx)
        }
    }

    fn row_ids(rows: &KeyedSubViews<u64, RowView>) -> Vec<(u64, EntityId)> {
        rows.keyed_iter()
            .map(|(key, view)| (*key, view.entity().entity_id()))
            .collect()
    }

    #[test]
    fn sync_reuses_entities_when_items_reorder_and_remove() {
        let renders = Rc::new(Cell::new(0));
        let mut app = TestApp::new();

        let mut rows = app.update({
            let renders = renders.clone();
            move |cx| {
                init(cx);
                let mut rows = KeyedSubViews::<u64, RowView>::new();
                rows.sync(
                    cx,
                    vec![
                        Item::new(1, "one"),
                        Item::new(2, "two"),
                        Item::new(3, "three"),
                    ],
                    |item| item.id,
                    |item, _cx| RowView::new(item, renders.clone()),
                    |item, row, _cx| row.update_item(item),
                );
                rows
            }
        });

        let first_ids = row_ids(&rows);

        rows = app.update({
            let renders = renders.clone();
            move |cx| {
                rows.sync(
                    cx,
                    vec![Item::new(3, "three"), Item::new(1, "one")],
                    |item| item.id,
                    |item, _cx| RowView::new(item, renders.clone()),
                    |item, row, _cx| row.update_item(item),
                );
                rows
            }
        });

        assert_eq!(
            row_ids(&rows),
            vec![(3, first_ids[2].1), (1, first_ids[0].1)]
        );
    }

    #[test]
    fn cached_rows_reuse_clean_siblings_when_one_item_changes() {
        let row_renders = Rc::new(Cell::new(0));
        let host_renders = Rc::new(Cell::new(0));

        let mut app = TestApp::new();
        let mut window = app.open_window({
            let row_renders = row_renders.clone();
            let host_renders = host_renders.clone();
            move |_, cx| ListHost::new(cx, row_renders, host_renders)
        });
        let root = window.root();

        window.draw();
        let initial_row_renders = row_renders.get();
        let initial_host_renders = host_renders.get();
        assert_eq!(initial_row_renders, 3);

        app.update_entity(&root, |host, cx| {
            host.items.set(
                cx,
                vec![
                    Item::new(1, "one"),
                    Item::new(2, "TWO"),
                    Item::new(3, "three"),
                ],
            );
        });
        window.draw();

        assert_eq!(row_renders.get(), initial_row_renders + 1);
        assert!(host_renders.get() > initial_host_renders);
    }
}
