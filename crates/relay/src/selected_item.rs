//! Selected item projections for selector-backed collections.

use std::hash::Hash;

use gpui::App;

use crate::{Memo, Selector, Signal};

/// Build a memoized selected item projection from a collection and a selector.
///
/// This is the small app-facing helper for the common shape:
/// `Signal<Vec<T>>` or `Memo<Vec<T>>` plus `Selector<K>` produces the currently
/// selected item as `Memo<Option<T>>`.
pub trait SelectedItemExt<T> {
    /// Derive the item matching the selector's current key.
    fn selected_by<K>(
        &self,
        cx: &mut App,
        selector: Selector<K>,
        key: impl Fn(&T) -> K + 'static,
    ) -> Memo<Option<T>>
    where
        T: Clone + PartialEq + 'static,
        K: Clone + Eq + Hash + PartialEq + 'static;

    /// Derive the selected item, falling back to the first item when the
    /// selector is empty or points at a missing key.
    fn selected_by_or_first<K>(
        &self,
        cx: &mut App,
        selector: Selector<K>,
        key: impl Fn(&T) -> K + 'static,
    ) -> Memo<Option<T>>
    where
        T: Clone + PartialEq + 'static,
        K: Clone + Eq + Hash + PartialEq + 'static;
}

impl<T> SelectedItemExt<T> for Signal<Vec<T>> {
    fn selected_by<K>(
        &self,
        cx: &mut App,
        selector: Selector<K>,
        key: impl Fn(&T) -> K + 'static,
    ) -> Memo<Option<T>>
    where
        T: Clone + PartialEq + 'static,
        K: Clone + Eq + Hash + PartialEq + 'static,
    {
        let items = self.clone();
        Memo::new(cx, move |cx| {
            let selected = selector.get(cx);
            items.read(cx, |items| {
                selected_item(items, selected.as_ref(), &key, MissingSelection::None)
            })
        })
    }

    fn selected_by_or_first<K>(
        &self,
        cx: &mut App,
        selector: Selector<K>,
        key: impl Fn(&T) -> K + 'static,
    ) -> Memo<Option<T>>
    where
        T: Clone + PartialEq + 'static,
        K: Clone + Eq + Hash + PartialEq + 'static,
    {
        let items = self.clone();
        Memo::new(cx, move |cx| {
            let selected = selector.get(cx);
            items.read(cx, |items| {
                selected_item(items, selected.as_ref(), &key, MissingSelection::First)
            })
        })
    }
}

impl<T> SelectedItemExt<T> for Memo<Vec<T>>
where
    T: Clone + PartialEq + 'static,
{
    fn selected_by<K>(
        &self,
        cx: &mut App,
        selector: Selector<K>,
        key: impl Fn(&T) -> K + 'static,
    ) -> Memo<Option<T>>
    where
        T: Clone + PartialEq + 'static,
        K: Clone + Eq + Hash + PartialEq + 'static,
    {
        let items = self.clone();
        Memo::new(cx, move |cx| {
            let selected = selector.get(cx);
            items.read(cx, |items| {
                selected_item(items, selected.as_ref(), &key, MissingSelection::None)
            })
        })
    }

    fn selected_by_or_first<K>(
        &self,
        cx: &mut App,
        selector: Selector<K>,
        key: impl Fn(&T) -> K + 'static,
    ) -> Memo<Option<T>>
    where
        T: Clone + PartialEq + 'static,
        K: Clone + Eq + Hash + PartialEq + 'static,
    {
        let items = self.clone();
        Memo::new(cx, move |cx| {
            let selected = selector.get(cx);
            items.read(cx, |items| {
                selected_item(items, selected.as_ref(), &key, MissingSelection::First)
            })
        })
    }
}

#[derive(Clone, Copy)]
enum MissingSelection {
    None,
    First,
}

fn selected_item<T, K>(
    items: &[T],
    selected: Option<&K>,
    key: &impl Fn(&T) -> K,
    missing: MissingSelection,
) -> Option<T>
where
    T: Clone,
    K: PartialEq,
{
    if let Some(selected) = selected
        && let Some(item) = items.iter().find(|item| key(item) == *selected)
    {
        return Some(item.clone());
    }

    match missing {
        MissingSelection::None => None,
        MissingSelection::First => items.first().cloned(),
    }
}

#[cfg(test)]
mod tests {
    use gpui::TestApp;

    use crate::{Memo, SelectedItemExt, Selector, Signal, init};

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
    fn signal_selected_by_tracks_selection_and_item_updates() {
        let mut app = TestApp::new();
        let (items, selector, selected) = app.update(|cx| {
            init(cx);
            let items = Signal::new(cx, vec![Item::new(1, "one"), Item::new(2, "two")]);
            let selector = Selector::new(cx, Some(1));
            let selected = items.selected_by(cx, selector.clone(), |item| item.id);
            (items, selector, selected)
        });

        let initial = app.read(|cx| selected.get(cx).map(|item| item.label));
        assert_eq!(initial, Some("one"));

        app.update(|cx| selector.select(cx, 2));
        let next = app.read(|cx| selected.get(cx).map(|item| item.label));
        assert_eq!(next, Some("two"));

        app.update(|cx| {
            items.update(cx, |items| {
                let Some(item) = items.iter_mut().find(|item| item.id == 2) else {
                    return false;
                };
                item.label = "updated";
                true
            });
        });
        let updated = app.read(|cx| selected.get(cx).map(|item| item.label));
        assert_eq!(updated, Some("updated"));
    }

    #[test]
    fn signal_selected_by_returns_none_when_key_is_missing() {
        let mut app = TestApp::new();
        let selected = app.update(|cx| {
            init(cx);
            let items = Signal::new(cx, vec![Item::new(1, "one")]);
            let selector = Selector::new(cx, Some(9));
            items.selected_by(cx, selector, |item| item.id)
        });

        let value = app.read(|cx| selected.get(cx));
        assert_eq!(value, None);
    }

    #[test]
    fn signal_selected_by_or_first_falls_back_to_first_item() {
        let mut app = TestApp::new();
        let selected = app.update(|cx| {
            init(cx);
            let items = Signal::new(cx, vec![Item::new(1, "one"), Item::new(2, "two")]);
            let selector = Selector::new(cx, Some(9));
            items.selected_by_or_first(cx, selector, |item| item.id)
        });

        let value = app.read(|cx| selected.get(cx).map(|item| item.id));
        assert_eq!(value, Some(1));
    }

    #[test]
    fn memo_selected_by_tracks_filtered_items() {
        let mut app = TestApp::new();
        let (query, selector, selected) = app.update(|cx| {
            init(cx);
            let source = Signal::new(
                cx,
                vec![
                    Item::new(1, "alpha"),
                    Item::new(2, "beta"),
                    Item::new(3, "alpine"),
                ],
            );
            let query = Signal::new(cx, "al");
            let filtered = Memo::new(cx, {
                let source = source.clone();
                let query = query.clone();
                move |cx| {
                    let query = query.get(cx);
                    source.read(cx, |items| {
                        items
                            .iter()
                            .filter(|item| item.label.contains(query))
                            .cloned()
                            .collect()
                    })
                }
            });
            let selector = Selector::new(cx, Some(3));
            let selected = filtered.selected_by(cx, selector.clone(), |item| item.id);
            (query, selector, selected)
        });

        let initial = app.read(|cx| selected.get(cx).map(|item| item.label));
        assert_eq!(initial, Some("alpine"));

        app.update(|cx| query.set(cx, "be"));
        let filtered_out = app.read(|cx| selected.get(cx));
        assert_eq!(filtered_out, None);

        app.update(|cx| selector.select(cx, 2));
        let selected_again = app.read(|cx| selected.get(cx).map(|item| item.label));
        assert_eq!(selected_again, Some("beta"));
    }
}
