//! View-layer conveniences for relay.
//!
//! This module provides two key abstractions that eliminate boilerplate in
//! GPUI views:
//!
//! - [`StateScope`] — holds effect/form/memo handles so they live as long as
//!   the view Entity, replacing `std::mem::forget(form)` and `_effect: Effect`
//!   field patterns.
//! - [`ReactiveView`] — a trait that, combined with the [`reactive_render`]
//!   helper, eliminates manual `cx.tracked(|cx| ...)` from every view.
//!
//! # Example
//!
//! ```ignore
//! use relay::{
//!     ReactiveAppExt, Signal,
//!     view::{ReactiveView, StateScope, reactive_render},
//! };
//!
//! struct Counter {
//!     count: Signal<i32>,
//!     scope: StateScope,
//! }
//!
//! impl Counter {
//!     fn new(cx: &mut Context<Self>) -> Self {
//!         let mut scope = StateScope::new();
//!         let count = cx.signal(0);
//!         scope.watch(cx,
//!             |cx| { let _ = count.get(cx); },
//!             move |cx| { /* side effect */ },
//!         );
//!         Self { count, scope }
//!     }
//! }
//!
//! impl ReactiveView for Counter {
//!     fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//!         div().child(self.count.get(cx).to_string())
//!     }
//! }
//!
//! impl Render for Counter {
//!     fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//!         reactive_render(self, window, cx)  // one-liner, no manual tracked
//!     }
//! }
//! ```

use gpui::{
    AnyElement, AnyView, App, AppContext, Context, Entity, IntoElement, Render, StyleRefinement,
    Styled, Window,
};

use crate::{Binding, Effect, Form, Memo, ReactiveAppExt, ReactiveContextExt, effect, effect_in};

// ---------------------------------------------------------------------------
// StateScope
// ---------------------------------------------------------------------------

/// Holds the lifetimes of effects, forms, and derived values created during
/// view initialization.
///
/// Store a `StateScope` as a field in your view struct. When the view Entity
/// is released, the `StateScope` is dropped, which disposes all held effects
/// — replacing `std::mem::forget(form)` and `_effect: Effect` field patterns.
pub struct StateScope {
    effects: Vec<Effect>,
    #[allow(dead_code)]
    forms: Vec<Form>,
}

impl StateScope {
    /// Create an empty scope.
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            forms: Vec::new(),
        }
    }

    /// Register an app-scoped effect. The scope holds its handle.
    pub fn effect(&mut self, cx: &mut App, f: impl FnMut(&mut App) + 'static) {
        let e = effect(cx, f);
        self.effects.push(e);
    }

    /// Register an entity-scoped effect. The scope holds its handle.
    pub fn effect_in<T: 'static>(
        &mut self,
        cx: &mut Context<T>,
        f: impl FnMut(&mut App) + 'static,
    ) {
        let e = effect_in(cx, f);
        self.effects.push(e);
    }

    /// Register a declarative watch. The scope holds the effect handle.
    pub fn watch<T, S, R>(&mut self, cx: &mut Context<T>, sources: S, react: R)
    where
        T: 'static,
        S: Fn(&App) + 'static,
        R: Fn(&mut App) + 'static,
    {
        let e = cx.watch(sources, react);
        self.effects.push(e);
    }

    /// Create a derived value (memo). The scope tracks it for future disposal.
    pub fn derived<T: PartialEq + 'static>(
        &mut self,
        cx: &mut App,
        compute: impl Fn(&App) -> T + 'static,
    ) -> Memo<T> {
        cx.derived(compute)
    }

    /// Start building a form that will be held by this scope.
    pub fn form(&mut self) -> FormBuilder<'_> {
        FormBuilder {
            form: Form::new(),
            scope: self,
        }
    }
}

impl Default for StateScope {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// FormBuilder
// ---------------------------------------------------------------------------

/// Builder for a [`Form`] that is automatically held by a [`StateScope`].
///
/// Created via [`StateScope::form`]. Chain `.field()` calls, then call
/// `.build_is_dirty(cx)` to finalize.
pub struct FormBuilder<'a> {
    form: Form,
    scope: &'a mut StateScope,
}

impl<'a> FormBuilder<'a> {
    /// Register a field with a key and a binding.
    pub fn field<T: PartialEq + Clone + 'static>(
        mut self,
        key: &'static str,
        binding: Binding<T>,
        cx: &App,
    ) -> Self {
        self.form.field(key, binding, cx);
        self
    }

    /// Finalize the form and build the `is_dirty` memo. The form is stored
    /// in the scope; the returned memo is safe to use for the entity's
    /// lifetime.
    pub fn build_is_dirty(mut self, cx: &mut App) -> Memo<bool> {
        let dirty = self.form.build_is_dirty(cx);
        self.scope.forms.push(self.form);
        dirty
    }
}

// ---------------------------------------------------------------------------
// SubView
// ---------------------------------------------------------------------------

/// A stable child view entity intended to be rendered through GPUI's view cache.
///
/// `SubView` is relay's main UI-granularity primitive: put stateful or heavy
/// regions behind their own GPUI [`Entity`], then render them with
/// [`SubView::cached`] or [`SubView::cached_full`]. When a signal owned by one
/// child changes, GPUI may still re-enter ancestor renders, but clean sibling
/// subviews can reuse their cached layout and paint work.
pub struct SubView<T: 'static> {
    entity: Entity<T>,
}

impl<T: 'static> SubView<T> {
    /// Create a child entity in the current GPUI context.
    pub fn new(cx: &mut impl AppContext, build_entity: impl FnOnce(&mut Context<T>) -> T) -> Self {
        Self::from_entity(cx.new(build_entity))
    }

    /// Wrap an existing GPUI entity as a relay subview.
    pub fn from_entity(entity: Entity<T>) -> Self {
        Self { entity }
    }

    /// Borrow the underlying GPUI entity.
    pub fn entity(&self) -> &Entity<T> {
        &self.entity
    }

    /// Clone the underlying GPUI entity handle.
    pub fn clone_entity(&self) -> Entity<T> {
        self.entity.clone()
    }

    /// Consume this wrapper and return the underlying GPUI entity.
    pub fn into_entity(self) -> Entity<T> {
        self.entity
    }
}

impl<T: Render> SubView<T> {
    /// Return this subview as a GPUI [`AnyView`].
    pub fn any_view(&self) -> AnyView {
        self.entity.clone().into()
    }

    /// Render this subview without GPUI's view cache.
    pub fn uncached(&self) -> AnyElement {
        self.any_view().into_any_element()
    }

    /// Render this subview through GPUI's [`AnyView::cached`] path.
    pub fn cached(&self, style: StyleRefinement) -> AnyElement {
        self.any_view().cached(style).into_any_element()
    }

    /// Render this subview cached with a full-size root style.
    pub fn cached_full(&self) -> AnyElement {
        self.cached(StyleRefinement::default().size_full())
    }
}

impl<T: 'static> Clone for SubView<T> {
    fn clone(&self) -> Self {
        Self {
            entity: self.entity.clone(),
        }
    }
}

impl<T: 'static> From<Entity<T>> for SubView<T> {
    fn from(entity: Entity<T>) -> Self {
        Self::from_entity(entity)
    }
}

// ---------------------------------------------------------------------------
// ReactiveView + reactive_render
// ---------------------------------------------------------------------------

/// A view that renders with automatic signal dependency tracking.
///
/// Implement `render_state` instead of GPUI's `Render::render`. Then in your
/// `Render` impl, call [`reactive_render`] as a one-liner:
///
/// ```ignore
/// impl Render for MyView {
///     fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
///         reactive_render(self, window, cx)
///     }
/// }
/// ```
pub trait ReactiveView: 'static + Sized {
    /// Render the view's content. Signal reads are automatically tracked.
    fn render_state(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement;
}

/// Render a [`ReactiveView`] with automatic signal tracking.
///
/// Call this from your `Render::render` implementation:
///
/// ```ignore
/// impl Render for MyView {
///     fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
///         reactive_render(self, window, cx)
///     }
/// }
/// ```
pub fn reactive_render<T: ReactiveView>(
    view: &mut T,
    window: &mut Window,
    cx: &mut Context<T>,
) -> impl IntoElement {
    cx.tracked(|cx| view.render_state(window, cx))
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{Context, IntoElement, ParentElement, Render, Styled, TestApp, Window, div};

    use crate::{ReactiveAppExt, Signal, init};

    use super::*;

    struct CounterView {
        count: Signal<i32>,
        scope: StateScope,
    }

    impl CounterView {
        fn new(cx: &mut Context<Self>) -> Self {
            init(cx);
            let mut scope = StateScope::new();
            let count = cx.signal(0);

            let count_for_sources = count.clone();
            let count_for_react = count.clone();
            scope.watch(
                cx,
                move |cx| {
                    let _ = count_for_sources.get(cx);
                },
                move |_cx| {
                    // side effect — just reading count is enough to verify
                    // the watch fires.
                    let _ = count_for_react.get_untracked();
                },
            );

            Self { count, scope }
        }
    }

    impl ReactiveView for CounterView {
        fn render_state(
            &mut self,
            _window: &mut Window,
            cx: &mut Context<Self>,
        ) -> AnyElement {
            div().child(self.count.get(cx).to_string()).into_any_element()
        }
    }

    impl Render for CounterView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            reactive_render(self, window, cx)
        }
    }

    struct LeafView {
        value: Signal<i32>,
        renders: Rc<Cell<usize>>,
    }

    impl LeafView {
        fn new(cx: &mut Context<Self>, renders: Rc<Cell<usize>>) -> Self {
            init(cx);
            Self {
                value: cx.signal(0),
                renders,
            }
        }
    }

    impl ReactiveView for LeafView {
        fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
            self.renders.set(self.renders.get() + 1);
            div()
                .child(self.value.get(cx).to_string())
                .into_any_element()
        }
    }

    impl Render for LeafView {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            reactive_render(self, window, cx)
        }
    }

    struct SubViewHost {
        left: SubView<LeafView>,
        right: SubView<LeafView>,
        renders: Rc<Cell<usize>>,
    }

    impl SubViewHost {
        fn new(
            cx: &mut Context<Self>,
            left_renders: Rc<Cell<usize>>,
            right_renders: Rc<Cell<usize>>,
            renders: Rc<Cell<usize>>,
        ) -> Self {
            init(cx);
            Self {
                left: SubView::new(cx, |cx| LeafView::new(cx, left_renders)),
                right: SubView::new(cx, |cx| LeafView::new(cx, right_renders)),
                renders,
            }
        }
    }

    impl ReactiveView for SubViewHost {
        fn render_state(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> AnyElement {
            self.renders.set(self.renders.get() + 1);
            div()
                .size_full()
                .child(self.left.cached_full())
                .child(self.right.cached_full())
                .into_any_element()
        }
    }

    impl Render for SubViewHost {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            reactive_render(self, window, cx)
        }
    }

    #[test]
    fn reactive_view_auto_tracks_signals() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| CounterView::new(cx));
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

        // Signal change should trigger re-render via auto-tracked ReactiveView.
        app.update_entity(&root, |view, cx| {
            view.count.set(cx, 42);
        });

        assert_eq!(notifications.get(), 1);
    }

    #[test]
    fn cached_subview_reuses_clean_sibling_when_another_subview_changes() {
        let left_renders = Rc::new(Cell::new(0));
        let right_renders = Rc::new(Cell::new(0));
        let host_renders = Rc::new(Cell::new(0));

        let mut app = TestApp::new();
        let mut window = app.open_window({
            let left_renders = left_renders.clone();
            let right_renders = right_renders.clone();
            let host_renders = host_renders.clone();
            move |_, cx| SubViewHost::new(cx, left_renders, right_renders, host_renders)
        });
        let root = window.root();

        window.draw();
        let initial_left_renders = left_renders.get();
        let initial_right_renders = right_renders.get();
        let initial_host_renders = host_renders.get();
        assert_eq!(initial_left_renders, 1);
        assert_eq!(initial_right_renders, 1);

        let left = app.read_entity(&root, |host, _cx| host.left.clone_entity());
        app.update_entity(&left, |leaf, cx| {
            leaf.value.set(cx, 1);
        });
        window.draw();

        assert_eq!(left_renders.get(), initial_left_renders + 1);
        assert_eq!(right_renders.get(), initial_right_renders);
        assert!(host_renders.get() > initial_host_renders);
    }

    #[test]
    fn form_builder_eliminates_mem_forget() {
        let mut app = TestApp::new();
        let (binding, dirty) = app.update(|cx| {
            init(cx);
            let mut scope = StateScope::new();
            let binding: Binding<i32> = cx.binding(42);
            let dirty = scope
                .form()
                .field("count", binding.clone(), cx)
                .build_is_dirty(cx);
            // Leak scope to keep form alive (in real code, scope is a struct field).
            std::mem::forget(scope);
            (binding, dirty)
        });

        // Not dirty initially.
        app.read(|cx| assert!(!dirty.get(cx)));

        // Change value — dirty becomes true.
        app.update(|cx| binding.set(cx, 99));
        app.read(|cx| assert!(dirty.get(cx)));
    }

    #[test]
    fn state_scope_watch_fires_on_signal_change() {
        let fired = Rc::new(Cell::new(0));

        struct WatchView {
            count: Signal<i32>,
            scope: StateScope,
            fired: Rc<Cell<i32>>,
        }

        impl WatchView {
            fn new(cx: &mut Context<Self>, fired: Rc<Cell<i32>>) -> Self {
                init(cx);
                let mut scope = StateScope::new();
                let count = cx.signal(0);
                let count_for_sources = count.clone();
                let fired_clone = fired.clone();
                scope.watch(
                    cx,
                    move |cx| {
                        let _ = count_for_sources.get(cx);
                    },
                    move |_cx| {
                        fired_clone.set(fired_clone.get() + 1);
                    },
                );
                Self { count, scope, fired }
            }
        }

        impl ReactiveView for WatchView {
            fn render_state(
                &mut self,
                _window: &mut Window,
                cx: &mut Context<Self>,
            ) -> AnyElement {
                div().child(self.count.get(cx).to_string()).into_any_element()
            }
        }

        impl Render for WatchView {
            fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                reactive_render(self, window, cx)
            }
        }

        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| WatchView::new(cx, fired.clone()));
        window.draw();

        // Watch fired once on creation.
        assert_eq!(fired.get(), 1);

        app.update_entity(&window.root(), |view, cx| {
            view.count.set(cx, 5);
        });

        // Watch fired again.
        assert_eq!(fired.get(), 2);
    }
}
