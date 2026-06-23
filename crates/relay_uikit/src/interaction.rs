//! View-free callback types for Relay components.
//!
//! Instead of GPUI's `Entity<X>`-coupled listeners, every component accepts plain
//! `Box<dyn Fn>` closures.  This keeps the component library decoupled from any
//! concrete app view and lets the gallery and the real workbench wire the same
//! component to different handlers.
//!
//! # Box vs Rc
//!
//! Use `Box` for single-consumption handlers (`RenderOnce` components that fire the
//! handler once per render).  Use `Rc` when a handler must be cloned into multiple
//! sub-elements (e.g. a split-pane handle that renders two drag zones).

use std::{hash::Hash, rc::Rc};

use gpui::{App, ClickEvent, KeyDownEvent, Window};
use relay::{Binding, MultiSelectionModel, OrderedSelectionModel, SelectionModel, Selector};

// ---------------------------------------------------------------------------
// Mouse click handlers
// ---------------------------------------------------------------------------

/// A simple click callback (`on_click`).  Receives the raw [`ClickEvent`].
pub type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Shared variant of [`ClickHandler`] for handlers that are cloned across
/// multiple elements.
pub type SharedClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

// ---------------------------------------------------------------------------
// Overlay / dialog lifecycle handlers
// ---------------------------------------------------------------------------

/// Fires when an overlay or dialog is dismissed (click-outside, Escape key).
pub type DismissHandler = Box<dyn Fn(&mut Window, &mut App) + 'static>;
pub type SharedDismissHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

/// Fires on form submit (Enter in a focused input, submit button).
pub type SubmitHandler = Box<dyn Fn(&mut Window, &mut App) + 'static>;
pub type SharedSubmitHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

/// Fires when an interaction is explicitly cancelled (Escape, cancel button).
pub type CancelHandler = Box<dyn Fn(&mut Window, &mut App) + 'static>;
pub type SharedCancelHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

// ---------------------------------------------------------------------------
// Open / expanded state
// ---------------------------------------------------------------------------

/// Type-erased adapter for host-owned open/closed state.
#[derive(Clone)]
pub struct OpenState {
    get: Rc<dyn Fn(&App) -> bool + 'static>,
    set: Rc<dyn Fn(&mut App, bool) + 'static>,
}

impl OpenState {
    /// Adapt a Relay boolean binding as an open-state source.
    pub fn binding(binding: Binding<bool>) -> Self {
        let read_binding = binding.clone();

        Self {
            get: Rc::new(move |cx| read_binding.get(cx)),
            set: Rc::new(move |cx, open| binding.set(cx, open)),
        }
    }

    /// Return whether the controlled surface is open.
    pub fn get(&self, cx: &App) -> bool {
        (self.get)(cx)
    }

    /// Set the controlled surface open state.
    pub fn set(&self, cx: &mut App, open: bool) {
        (self.set)(cx, open);
    }

    /// Toggle the controlled surface open state.
    pub fn toggle(&self, cx: &mut App) {
        self.set(cx, !self.get(cx));
    }

    /// Close the controlled surface.
    pub fn close(&self, cx: &mut App) {
        self.set(cx, false);
    }
}

// ---------------------------------------------------------------------------
// Selection handlers
// ---------------------------------------------------------------------------

/// Type-erased adapter from Relay's keyed [`Selector`] to boolean component
/// selection state.
#[derive(Clone)]
pub struct SelectionBinding {
    is_selected: Rc<dyn Fn(&mut App) -> bool + 'static>,
    select: Rc<dyn Fn(&mut App) + 'static>,
}

impl SelectionBinding {
    /// Bind one component instance to `key` in a Relay binding.
    pub fn binding<K>(binding: Binding<K>, key: K) -> Self
    where
        K: Clone + Eq + Hash + PartialEq + 'static,
    {
        let read_binding = binding.clone();
        let read_key = key.clone();

        Self {
            is_selected: Rc::new(move |cx| read_binding.get(cx) == read_key.clone()),
            select: Rc::new(move |cx| binding.set(cx, key.clone())),
        }
    }

    /// Bind one component instance to `key` in a Relay selector.
    pub fn selector<K>(selector: Selector<K>, key: K) -> Self
    where
        K: Clone + Eq + Hash + PartialEq + 'static,
    {
        let read_selector = selector.clone();
        let read_key = key.clone();

        Self {
            is_selected: Rc::new(move |cx| read_selector.is_selected(cx, read_key.clone())),
            select: Rc::new(move |cx| selector.select(cx, key.clone())),
        }
    }

    /// Bind one component instance to `key` in a Relay selection model.
    pub fn selection_model<K>(selection: SelectionModel<K>, key: K) -> Self
    where
        K: Clone + Eq + Hash + PartialEq + 'static,
    {
        Self::selector(selection.selector().clone(), key)
    }

    /// Bind one component instance to `key` in a Relay ordered selection model.
    pub fn ordered_selection_model<K>(selection: OrderedSelectionModel<K>, key: K) -> Self
    where
        K: Clone + Eq + Hash + PartialEq + 'static,
    {
        Self::selection_model(selection.selection().clone(), key)
    }

    /// Bind one component instance to `key` as the active entry in a Relay multi-selection model.
    pub fn multi_selection_active<K>(selection: MultiSelectionModel<K>, key: K) -> Self
    where
        K: Clone + Eq + Hash + PartialEq + 'static,
    {
        let read_selection = selection.clone();
        let read_key = key.clone();

        Self {
            is_selected: Rc::new(move |cx| read_selection.is_active(cx, read_key.clone())),
            select: Rc::new(move |cx| {
                let _ = selection.select_only(cx, key.clone());
            }),
        }
    }

    /// Return whether the bound key is selected.
    pub fn is_selected(&self, cx: &mut App) -> bool {
        (self.is_selected)(cx)
    }

    /// Select the bound key.
    pub fn select(&self, cx: &mut App) {
        (self.select)(cx);
    }
}

/// Type-erased adapter for components that read and write a single selected key.
#[derive(Clone)]
pub struct SelectionSource<K> {
    get: Rc<dyn Fn(&App) -> Option<K> + 'static>,
    select: Rc<dyn Fn(&mut App, K) + 'static>,
}

impl<K> SelectionSource<K>
where
    K: Clone + Eq + Hash + PartialEq + 'static,
{
    /// Adapt a Relay binding as a selected-key source.
    pub fn binding(binding: Binding<K>) -> Self {
        let read_binding = binding.clone();

        Self {
            get: Rc::new(move |cx| Some(read_binding.get(cx))),
            select: Rc::new(move |cx, key| binding.set(cx, key)),
        }
    }

    /// Adapt a Relay selector as a selected-key source.
    pub fn selector(selector: Selector<K>) -> Self {
        let read_selector = selector.clone();

        Self {
            get: Rc::new(move |cx| read_selector.get(cx)),
            select: Rc::new(move |cx, key| selector.select(cx, key)),
        }
    }

    /// Adapt a Relay selection model as a selected-key source.
    pub fn selection_model(selection: SelectionModel<K>) -> Self {
        Self::selector(selection.selector().clone())
    }

    /// Adapt a Relay ordered selection model as a selected-key source.
    pub fn ordered_selection_model(selection: OrderedSelectionModel<K>) -> Self {
        Self::selection_model(selection.selection().clone())
    }

    /// Adapt the active key of a Relay multi-selection model as a selected-key source.
    pub fn multi_selection_active(selection: MultiSelectionModel<K>) -> Self {
        let read_selection = selection.clone();

        Self {
            get: Rc::new(move |cx| read_selection.active(cx)),
            select: Rc::new(move |cx, key| {
                let _ = selection.select_only(cx, key);
            }),
        }
    }

    /// Return the currently selected key.
    pub fn get(&self, cx: &App) -> Option<K> {
        (self.get)(cx)
    }

    /// Select the given key.
    pub fn select(&self, cx: &mut App, key: K) {
        (self.select)(cx, key);
    }
}

// ---------------------------------------------------------------------------
// Generic action / change handlers
// ---------------------------------------------------------------------------

/// Generic action dispatch.  Use when a component emits a single action value
/// (e.g. a button press with associated data).
pub type ActionHandler<T> = Box<dyn Fn(T, &mut Window, &mut App) + 'static>;
pub type SharedActionHandler<T> = Rc<dyn Fn(T, &mut Window, &mut App) + 'static>;

/// Value-change notification.  Fires whenever the host-owned state is mutated
/// (text changed, slider moved, toggle flipped).
pub type ChangeHandler<T> = Box<dyn Fn(T, &mut Window, &mut App) + 'static>;
pub type SharedChangeHandler<T> = Rc<dyn Fn(T, &mut Window, &mut App) + 'static>;

// ---------------------------------------------------------------------------
// Keyboard handlers
// ---------------------------------------------------------------------------

/// Fires on every `keydown` event when the element has focus.
pub type KeyHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;

/// Like [`KeyHandler`], but returns `true` when the event was consumed
/// (stops propagation).  Returning `false` lets the event bubble.
pub type KeyCaptureHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) -> bool + 'static>;

// ---------------------------------------------------------------------------
// Color picker handler
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use gpui::{AppContext, Context, TestApp};
    use relay::{
        MultiSelectionModel, OrderedSelectionModel, ReactiveAppExt, SelectionModel,
        SelectionReconcilePolicy, Signal, init, use_multi_selection_model,
        use_ordered_selection_model,
    };

    use super::{OpenState, SelectionBinding, SelectionSource};

    #[test]
    fn selection_binding_selects_selector_key() {
        let mut app = TestApp::new();
        let (first, second) = app.update(|cx| {
            init(cx);
            let selector = cx.selector(Some("first"));
            (
                SelectionBinding::selector(selector.clone(), "first"),
                SelectionBinding::selector(selector, "second"),
            )
        });

        app.update(|cx| {
            assert!(first.is_selected(cx));
            assert!(!second.is_selected(cx));

            second.select(cx);

            assert!(!first.is_selected(cx));
            assert!(second.is_selected(cx));
        });
    }

    #[test]
    fn selection_binding_selection_model_selects_model_key() {
        let mut app = TestApp::new();
        let (selection, bound) = app.update(|cx| {
            init(cx);
            let selection = SelectionModel::new(cx, Some("open"));
            let bound = SelectionBinding::selection_model(selection.clone(), "close");
            (selection, bound)
        });

        app.update(|cx| {
            assert!(!bound.is_selected(cx));

            bound.select(cx);

            assert!(bound.is_selected(cx));
        });

        app.read(|cx| {
            assert_eq!(selection.get(cx), Some("close"));
        });
    }

    #[test]
    fn selection_source_selection_model_reads_and_selects_model_key() {
        let mut app = TestApp::new();
        let (selection, source) = app.update(|cx| {
            init(cx);
            let selection = SelectionModel::new(cx, Some("open"));
            let source = SelectionSource::selection_model(selection.clone());
            (selection, source)
        });

        app.read(|cx| {
            assert_eq!(source.get(cx), Some("open"));
        });

        app.update(|cx| {
            source.select(cx, "close");
        });

        app.read(|cx| {
            assert_eq!(selection.get(cx), Some("close"));
            assert_eq!(source.get(cx), Some("close"));
        });
    }

    #[test]
    fn selection_binding_ordered_selection_model_uses_underlying_selector() {
        struct Host {
            selection: OrderedSelectionModel<&'static str>,
        }

        impl Host {
            fn new(cx: &mut Context<Self>) -> Self {
                init(cx);
                let keys = Signal::new(cx, vec!["open", "close"]);
                let keys_for_selection = keys.clone();
                let selection = use_ordered_selection_model(
                    cx,
                    Some("open"),
                    move |cx| keys_for_selection.get(cx),
                    SelectionReconcilePolicy::SelectFirst,
                );
                Self { selection }
            }
        }

        let mut app = TestApp::new();
        let (selection, bound) = app.update(|cx| {
            let host = cx.new(Host::new);
            let selection = host.read(cx).selection.clone();
            let bound = SelectionBinding::ordered_selection_model(selection.clone(), "close");
            (selection, bound)
        });

        app.update(|cx| {
            assert!(!bound.is_selected(cx));

            bound.select(cx);

            assert!(bound.is_selected(cx));
        });

        app.read(|cx| {
            assert_eq!(selection.get(cx), Some("close"));
        });
    }

    #[test]
    fn selection_source_multi_selection_active_reads_and_selects_active_key() {
        struct Host {
            selection: MultiSelectionModel<&'static str>,
        }

        impl Host {
            fn new(cx: &mut Context<Self>) -> Self {
                init(cx);
                let keys = Signal::new(cx, vec!["open", "close", "review"]);
                let keys_for_selection = keys.clone();
                let selection = use_multi_selection_model(
                    cx,
                    Some("open"),
                    ["open"],
                    move |cx| keys_for_selection.get(cx),
                    SelectionReconcilePolicy::SelectFirst,
                );
                Self { selection }
            }
        }

        let mut app = TestApp::new();
        let (selection, source, bound) = app.update(|cx| {
            let host = cx.new(Host::new);
            let selection = host.read(cx).selection.clone();
            let source = SelectionSource::multi_selection_active(selection.clone());
            let bound = SelectionBinding::multi_selection_active(selection.clone(), "review");
            (selection, source, bound)
        });

        app.read(|cx| {
            assert_eq!(source.get(cx), Some("open"));
        });

        app.update(|cx| {
            assert!(!bound.is_selected(cx));

            source.select(cx, "close");
            assert_eq!(source.get(cx), Some("close"));

            bound.select(cx);
            assert!(bound.is_selected(cx));
        });

        app.read(|cx| {
            assert_eq!(selection.active(cx), Some("review"));
            assert_eq!(selection.selected_keys(cx), vec!["review"]);
        });
    }

    #[test]
    fn open_state_binding_reads_and_toggles_value() {
        let mut app = TestApp::new();
        let open = app.update(|cx| {
            init(cx);
            OpenState::binding(cx.binding(false))
        });

        app.read(|cx| {
            assert!(!open.get(cx));
        });

        app.update(|cx| {
            open.toggle(cx);
            assert!(open.get(cx));

            open.close(cx);
            assert!(!open.get(cx));
        });
    }
}
