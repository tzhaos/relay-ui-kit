use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled,
    Window, div,
};
use relay::Signal;

type ItemKeyFn<T> = dyn Fn(&T) -> usize + 'static;
type ItemRenderFn<T> = dyn Fn(&T, &mut Window, &mut App) -> AnyElement + 'static;

/// A lightweight reactive element-list renderer backed by a `Signal<Vec<T>>`.
///
/// `ForEach` reads the signal during render (subscribing the surrounding view)
/// and maps each item to an element via the provided `render` closure. The
/// `key` closure produces a stable numeric key per item, which is combined
/// with this component's id to create stable per-item element ids.
///
/// Use this instead of `div().children(...)` when the list comes from a
/// [`Signal`] and you want the view to refresh automatically when items are
/// added, removed, or reordered. The surrounding view still rebuilds these
/// child elements during render, so this is best for cheap, stateless rows.
///
/// When each row has its own state, subscriptions, async resources, or enough
/// rendering work that clean sibling rows should reuse GPUI's view cache, store
/// `relay::KeyedSubViews` in the host entity instead.
///
/// # Example
///
/// ```ignore
/// ForEach::new("todos", todos_signal)
///     .key(|todo| todo.id as usize)
///     .render_item(|todo| TodoRow::new(todo).into_any_element())
/// ```
#[derive(IntoElement)]
pub struct ForEach<T: Clone + 'static> {
    id: ElementId,
    signal: Signal<Vec<T>>,
    key: Option<Rc<ItemKeyFn<T>>>,
    render: Option<Rc<ItemRenderFn<T>>>,
}

impl<T: Clone + 'static> ForEach<T> {
    /// Create a `ForEach` bound to a list signal.
    pub fn new(id: impl Into<ElementId>, signal: Signal<Vec<T>>) -> Self {
        Self {
            id: id.into(),
            signal,
            key: None,
            render: None,
        }
    }

    /// Provide a stable numeric key for each item.
    ///
    /// The key is combined with the `ForEach` id to form a stable per-item
    /// element id. If omitted, items are keyed by their index in the current
    /// list, which is fine for append-only lists but unstable under reorder.
    pub fn key(mut self, key: impl Fn(&T) -> usize + 'static) -> Self {
        self.key = Some(Rc::new(key));
        self
    }

    /// Provide the render closure that maps each item to an element.
    pub fn render_item(
        mut self,
        render: impl Fn(&T, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.render = Some(Rc::new(render));
        self
    }
}

impl<T: Clone + 'static> RenderOnce for ForEach<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let items = self.signal.get(cx);
        let key_fn = self.key;
        let render_fn = self.render;
        let id = self.id;

        let mut container = div().id(id.clone()).flex().flex_col();

        let Some(render_fn) = render_fn else {
            return container;
        };

        for (index, item) in items.iter().enumerate() {
            // Use the caller-provided key to build a per-item element id, or
            // fall back to the index for append-only lists. We combine the
            // parent id and the numeric key into a string id so GPUI can diff
            // stably across renders.
            let item_key = match &key_fn {
                Some(key_fn) => key_fn(item),
                None => index,
            };
            let element = render_fn(item, window, cx);
            let child_id = gpui::SharedString::from(format!("{id:?}-{item_key}"));
            container = container.child(div().id(child_id).child(element));
        }

        container
    }
}
