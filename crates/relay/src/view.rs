//! View-layer conveniences for relay.
//!
//! This module provides two key abstractions that eliminate boilerplate in
//! GPUI views:
//!
//! - [`StateScope`] — holds entity-scoped effect handles and forms in the view
//!   struct, replacing `std::mem::forget(form)` and `_effect: Effect` field
//!   patterns.
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

use std::{
    cell::{Cell, RefCell},
    future::Future,
    rc::Rc,
};

use gpui::{
    AnyElement, AnyView, App, AppContext, AsyncApp, Context, Entity, IntoElement, Render,
    StyleRefinement, Styled, Window,
};

use crate::{
    Binding, CleanupScope, Effect, Form, Memo, ReactiveAppExt, ReactiveContextExt, Resource,
    effect, effect_in, effect_in_with_cleanup,
};

// ---------------------------------------------------------------------------
// StateScope
// ---------------------------------------------------------------------------

/// Holds entity-owned effects and forms created during view initialization.
///
/// Store a `StateScope` as a field in your view struct when effects, resource
/// source watchers, or dirty-check-only forms should live with that GPUI
/// entity. Entity-scoped effect helpers register GPUI release cleanup; the
/// scope keeps their handles reachable for the same lifetime.
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

    /// Register an app-scoped effect and retain its handle.
    ///
    /// Prefer [`effect`] plus an explicit [`Effect`] handle for app lifetime
    /// work, or [`StateScope::effect_in`] for view-owned work. App-scoped
    /// effects are not disposed by GPUI entity release.
    #[deprecated(
        note = "StateScope cannot dispose app-scoped effects on drop; use effect(...) with an explicit Effect handle, or use StateScope::effect_in for entity-scoped lifetime"
    )]
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

    /// Register an entity-scoped effect with per-run cleanup.
    pub fn effect_in_with_cleanup<T: 'static>(
        &mut self,
        cx: &mut Context<T>,
        f: impl FnMut(&mut App, &mut CleanupScope) + 'static,
    ) {
        let e = effect_in_with_cleanup(cx, f);
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

    /// Register a declarative watch that skips the initial reaction.
    pub fn watch_changes<T, S, R>(&mut self, cx: &mut Context<T>, sources: S, react: R)
    where
        T: 'static,
        S: Fn(&App) + 'static,
        R: Fn(&mut App) + 'static,
    {
        let e = cx.watch_changes(sources, react);
        self.effects.push(e);
    }

    /// Reload a resource when declared sources change.
    ///
    /// The first run only records sources, matching [`StateScope::watch_changes`].
    /// `build_load` runs on the GPUI app thread after a source change so callers
    /// can snapshot current signal values before handing async work to
    /// [`Resource::reload`].
    pub fn reload_resource_on_changes<T, Value, Error, Sources, BuildLoad, Load, Fut>(
        &mut self,
        cx: &mut Context<T>,
        resource: Resource<Value, Error>,
        sources: Sources,
        build_load: BuildLoad,
    ) where
        T: 'static,
        Value: 'static,
        Error: 'static,
        Sources: Fn(&App) + 'static,
        BuildLoad: Fn(&mut App) -> Load + 'static,
        Load: FnOnce(AsyncApp) -> Fut + 'static,
        Fut: Future<Output = Result<Value, Error>> + 'static,
    {
        self.watch_changes(cx, sources, move |cx| {
            let load = build_load(cx);
            resource.reload(cx, load);
        });
    }

    /// Reload a resource from a tracked source snapshot when that source changes.
    ///
    /// The `source` closure runs inside the tracked source phase and returns
    /// the exact value handed to `build_load` in the untracked reaction phase.
    /// Use this when the async load inputs are the declared sources; use
    /// [`StateScope::reload_resource_on_changes`] when source declaration and
    /// load construction intentionally differ.
    pub fn reload_resource_from_source<T, Value, Error, Source, SourceFn, BuildLoad, Load, Fut>(
        &mut self,
        cx: &mut Context<T>,
        resource: Resource<Value, Error>,
        source: SourceFn,
        build_load: BuildLoad,
    ) where
        T: 'static,
        Value: 'static,
        Error: 'static,
        Source: 'static,
        SourceFn: Fn(&App) -> Source + 'static,
        BuildLoad: Fn(Source) -> Load + 'static,
        Load: FnOnce(AsyncApp) -> Fut + 'static,
        Fut: Future<Output = Result<Value, Error>> + 'static,
    {
        let latest_source: Rc<RefCell<Option<Source>>> = Rc::new(RefCell::new(None));
        let latest_source_for_sources = latest_source.clone();
        let initialized = Cell::new(false);

        self.watch(
            cx,
            move |cx| {
                *latest_source_for_sources.borrow_mut() = Some(source(cx));
            },
            move |cx| {
                let Some(source) = latest_source.borrow_mut().take() else {
                    return;
                };
                if initialized.replace(true) {
                    let load = build_load(source);
                    resource.reload(cx, load);
                }
            },
        );
    }

    /// Load a resource immediately, then reload it when declared sources change.
    ///
    /// This is the source-driven helper for resources that start without a
    /// ready value. The first run records sources and calls [`Resource::load`];
    /// later source changes call [`Resource::reload`] so the last ready value
    /// remains visible during refresh.
    pub fn load_resource_on_changes<T, Value, Error, Sources, BuildLoad, Load, Fut>(
        &mut self,
        cx: &mut Context<T>,
        resource: Resource<Value, Error>,
        sources: Sources,
        build_load: BuildLoad,
    ) where
        T: 'static,
        Value: 'static,
        Error: 'static,
        Sources: Fn(&App) + 'static,
        BuildLoad: Fn(&mut App) -> Load + 'static,
        Load: FnOnce(AsyncApp) -> Fut + 'static,
        Fut: Future<Output = Result<Value, Error>> + 'static,
    {
        let initialized = Cell::new(false);
        self.watch(cx, sources, move |cx| {
            let load = build_load(cx);
            if initialized.replace(true) {
                resource.reload(cx, load);
            } else {
                resource.load(cx, load);
            }
        });
    }

    /// Load a resource from a tracked source snapshot, then reload on changes.
    ///
    /// The first run records the source and calls [`Resource::load`]; later
    /// source changes call [`Resource::reload`] with the latest source snapshot.
    pub fn load_resource_from_source<T, Value, Error, Source, SourceFn, BuildLoad, Load, Fut>(
        &mut self,
        cx: &mut Context<T>,
        resource: Resource<Value, Error>,
        source: SourceFn,
        build_load: BuildLoad,
    ) where
        T: 'static,
        Value: 'static,
        Error: 'static,
        Source: 'static,
        SourceFn: Fn(&App) -> Source + 'static,
        BuildLoad: Fn(Source) -> Load + 'static,
        Load: FnOnce(AsyncApp) -> Fut + 'static,
        Fut: Future<Output = Result<Value, Error>> + 'static,
    {
        let latest_source: Rc<RefCell<Option<Source>>> = Rc::new(RefCell::new(None));
        let latest_source_for_sources = latest_source.clone();
        let initialized = Cell::new(false);

        self.watch(
            cx,
            move |cx| {
                *latest_source_for_sources.borrow_mut() = Some(source(cx));
            },
            move |cx| {
                let Some(source) = latest_source.borrow_mut().take() else {
                    return;
                };
                let load = build_load(source);
                if initialized.replace(true) {
                    resource.reload(cx, load);
                } else {
                    resource.load(cx, load);
                }
            },
        );
    }

    /// Create a derived value (memo).
    ///
    /// The returned [`Memo`] owns its effect handle. Store it in the view if it
    /// should live for the view's lifetime.
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
    fn render_state(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement;
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
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
        time::Duration,
    };

    use gpui::{
        AppContext, Context, IntoElement, ParentElement, Render, Styled, TestApp, Window, div,
    };

    use crate::{ReactiveAppExt, Resource, ResourceState, Signal, init};

    use super::*;

    struct CounterView {
        count: Signal<i32>,
        _scope: StateScope,
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

            Self {
                count,
                _scope: scope,
            }
        }
    }

    impl ReactiveView for CounterView {
        fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
            div()
                .child(self.count.get(cx).to_string())
                .into_any_element()
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
    fn form_builder_keeps_form_in_view_scope() {
        struct FormView {
            binding: Binding<i32>,
            dirty: Memo<bool>,
            _scope: StateScope,
        }

        impl FormView {
            fn new(cx: &mut Context<Self>) -> Self {
                init(cx);
                let mut scope = StateScope::new();
                let binding: Binding<i32> = cx.binding(42);
                let dirty = scope
                    .form()
                    .field("count", binding.clone(), cx)
                    .build_is_dirty(cx);

                Self {
                    binding,
                    dirty,
                    _scope: scope,
                }
            }
        }

        impl ReactiveView for FormView {
            fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
                div()
                    .child(self.dirty.get(cx).to_string())
                    .into_any_element()
            }
        }

        impl Render for FormView {
            fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                reactive_render(self, window, cx)
            }
        }

        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| FormView::new(cx));
        let root = window.root();
        window.draw();

        let initial = app.update_entity(&root, |view, cx| view.dirty.get(cx));
        assert!(!initial);

        app.update_entity(&root, |view, cx| {
            view.binding.set(cx, 99);
        });

        let dirty = app.update_entity(&root, |view, cx| view.dirty.get(cx));
        assert!(dirty);
    }

    #[test]
    fn state_scope_watch_fires_on_signal_change() {
        let fired = Rc::new(Cell::new(0));

        struct WatchView {
            count: Signal<i32>,
            _scope: StateScope,
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
                Self {
                    count,
                    _scope: scope,
                }
            }
        }

        impl ReactiveView for WatchView {
            fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
                div()
                    .child(self.count.get(cx).to_string())
                    .into_any_element()
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

    #[test]
    fn state_scope_watch_changes_skips_initial_reaction() {
        let fired = Rc::new(Cell::new(0));

        struct WatchView {
            count: Signal<i32>,
            _scope: StateScope,
        }

        impl WatchView {
            fn new(cx: &mut Context<Self>, fired: Rc<Cell<i32>>) -> Self {
                init(cx);
                let mut scope = StateScope::new();
                let count = cx.signal(0);
                let count_for_sources = count.clone();
                scope.watch_changes(
                    cx,
                    move |cx| {
                        let _ = count_for_sources.get(cx);
                    },
                    move |_cx| {
                        fired.set(fired.get() + 1);
                    },
                );
                Self {
                    count,
                    _scope: scope,
                }
            }
        }

        impl ReactiveView for WatchView {
            fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
                div()
                    .child(self.count.get(cx).to_string())
                    .into_any_element()
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

        assert_eq!(fired.get(), 0);

        app.update_entity(&window.root(), |view, cx| {
            view.count.set(cx, 5);
        });

        assert_eq!(fired.get(), 1);
    }

    #[test]
    fn state_scope_effect_in_with_cleanup_runs_cleanup_on_entity_release() {
        struct ScopedCleanupView {
            _scope: StateScope,
        }

        impl ScopedCleanupView {
            fn new(cx: &mut Context<Self>, cleanup_count: Rc<Cell<i32>>) -> Self {
                init(cx);
                let mut scope = StateScope::new();
                scope.effect_in_with_cleanup(cx, move |_cx, cleanup| {
                    let cleanup_count = cleanup_count.clone();
                    cleanup.on_cleanup(move |_cx| {
                        cleanup_count.set(cleanup_count.get() + 1);
                    });
                });
                Self { _scope: scope }
            }
        }

        let mut app = TestApp::new();
        let cleanup_count = Rc::new(Cell::new(0));
        let weak = app.update({
            let cleanup_count = cleanup_count.clone();
            move |cx| {
                let entity = cx.new(|cx| ScopedCleanupView::new(cx, cleanup_count));
                let weak = entity.downgrade();
                drop(entity);
                weak
            }
        });

        weak.assert_released();
        assert_eq!(cleanup_count.get(), 1);
    }

    #[test]
    fn state_scope_reload_resource_from_source_uses_changed_source_snapshot() {
        struct SourceReloadView {
            source: Signal<i32>,
            resource: Resource<i32, &'static str>,
            _scope: StateScope,
        }

        impl SourceReloadView {
            fn new(cx: &mut Context<Self>, load_inputs: Rc<RefCell<Vec<i32>>>) -> Self {
                init(cx);
                let mut scope = StateScope::new();
                let source = cx.signal(0);
                let resource = cx.ready_resource::<_, &'static str>(10);
                let source_for_scope = source.clone();

                scope.reload_resource_from_source(
                    cx,
                    resource.clone(),
                    move |cx| source_for_scope.get(cx),
                    move |value| {
                        load_inputs.borrow_mut().push(value);
                        move |cx| async move {
                            cx.background_executor()
                                .timer(Duration::from_millis(20))
                                .await;
                            Ok(value)
                        }
                    },
                );

                Self {
                    source,
                    resource,
                    _scope: scope,
                }
            }
        }

        let load_inputs = Rc::new(RefCell::new(Vec::new()));
        let mut app = TestApp::new();
        let entity = app.update({
            let load_inputs = load_inputs.clone();
            move |cx| cx.new(|cx| SourceReloadView::new(cx, load_inputs))
        });

        assert!(load_inputs.borrow().is_empty());

        app.update_entity(&entity, |view, cx| {
            view.source.set(cx, 1);
        });

        assert_eq!(&*load_inputs.borrow(), &[1]);
        let state = app.update_entity(&entity, |view, cx| view.resource.get(cx));
        assert_eq!(state, ResourceState::Reloading(10));
    }

    #[test]
    fn state_scope_load_resource_from_source_loads_initially_then_reloads() {
        struct SourceLoadView {
            source: Signal<i32>,
            resource: Resource<i32, &'static str>,
            _scope: StateScope,
        }

        impl SourceLoadView {
            fn new(cx: &mut Context<Self>, load_inputs: Rc<RefCell<Vec<i32>>>) -> Self {
                init(cx);
                let mut scope = StateScope::new();
                let source = cx.signal(0);
                let resource = cx.pending_resource::<i32, &'static str>();
                let source_for_scope = source.clone();

                scope.load_resource_from_source(
                    cx,
                    resource.clone(),
                    move |cx| source_for_scope.get(cx),
                    move |value| {
                        load_inputs.borrow_mut().push(value);
                        move |cx| async move {
                            cx.background_executor()
                                .timer(Duration::from_millis(20))
                                .await;
                            Ok(value)
                        }
                    },
                );

                Self {
                    source,
                    resource,
                    _scope: scope,
                }
            }
        }

        let load_inputs = Rc::new(RefCell::new(Vec::new()));
        let mut app = TestApp::new();
        let entity = app.update({
            let load_inputs = load_inputs.clone();
            move |cx| cx.new(|cx| SourceLoadView::new(cx, load_inputs))
        });

        assert_eq!(&*load_inputs.borrow(), &[0]);
        let initial = app.update_entity(&entity, |view, cx| view.resource.get(cx));
        assert_eq!(initial, ResourceState::Pending);

        app.update_entity(&entity, |view, cx| {
            view.resource.set_ready(cx, 20);
            view.source.set(cx, 1);
        });

        assert_eq!(&*load_inputs.borrow(), &[0, 1]);
        let reloading = app.update_entity(&entity, |view, cx| view.resource.get(cx));
        assert_eq!(reloading, ResourceState::Reloading(20));
    }

    #[test]
    fn state_scope_reload_resource_on_changes_retains_latest_after_source_change() {
        struct ResourceReloadView {
            source: Signal<i32>,
            resource: Resource<i32, &'static str>,
            _scope: StateScope,
        }

        impl ResourceReloadView {
            fn new(cx: &mut Context<Self>) -> Self {
                init(cx);
                let mut scope = StateScope::new();
                let source = cx.signal(0);
                let resource = cx.ready_resource::<_, &'static str>(10);
                let source_for_sources = source.clone();
                let source_for_load = source.clone();

                scope.reload_resource_on_changes(
                    cx,
                    resource.clone(),
                    move |cx| {
                        let _ = source_for_sources.get(cx);
                    },
                    move |cx| {
                        let value = source_for_load.get(cx);
                        move |cx| async move {
                            cx.background_executor()
                                .timer(Duration::from_millis(20))
                                .await;
                            Ok(value)
                        }
                    },
                );

                Self {
                    source,
                    resource,
                    _scope: scope,
                }
            }
        }

        impl ReactiveView for ResourceReloadView {
            fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
                div()
                    .child(self.source.get(cx).to_string())
                    .into_any_element()
            }
        }

        impl Render for ResourceReloadView {
            fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                reactive_render(self, window, cx)
            }
        }

        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| ResourceReloadView::new(cx));
        let root = window.root();
        window.draw();

        let initial = app.update_entity(&root, |view, cx| view.resource.get(cx));
        assert_eq!(initial, ResourceState::Ready(10));

        app.update_entity(&root, |view, cx| {
            view.source.set(cx, 1);
        });

        let reloading = app.update_entity(&root, |view, cx| view.resource.get(cx));
        assert_eq!(reloading, ResourceState::Reloading(10));
    }

    #[test]
    fn state_scope_load_resource_on_changes_loads_initially_then_reloads() {
        struct ResourceLoadView {
            source: Signal<i32>,
            resource: Resource<i32, &'static str>,
            _scope: StateScope,
        }

        impl ResourceLoadView {
            fn new(cx: &mut Context<Self>) -> Self {
                init(cx);
                let mut scope = StateScope::new();
                let source = cx.signal(0);
                let resource = cx.pending_resource::<i32, &'static str>();
                let source_for_sources = source.clone();
                let source_for_load = source.clone();

                scope.load_resource_on_changes(
                    cx,
                    resource.clone(),
                    move |cx| {
                        let _ = source_for_sources.get(cx);
                    },
                    move |cx| {
                        let value = source_for_load.get(cx);
                        move |cx| async move {
                            cx.background_executor()
                                .timer(Duration::from_millis(20))
                                .await;
                            Ok(value)
                        }
                    },
                );

                Self {
                    source,
                    resource,
                    _scope: scope,
                }
            }
        }

        impl ReactiveView for ResourceLoadView {
            fn render_state(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
                div()
                    .child(self.source.get(cx).to_string())
                    .into_any_element()
            }
        }

        impl Render for ResourceLoadView {
            fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                reactive_render(self, window, cx)
            }
        }

        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| ResourceLoadView::new(cx));
        let root = window.root();
        window.draw();

        let initial = app.update_entity(&root, |view, cx| view.resource.get(cx));
        assert_eq!(initial, ResourceState::Pending);

        app.update_entity(&root, |view, cx| {
            view.resource.set_ready(cx, 20);
            view.source.set(cx, 1);
        });

        let reloading = app.update_entity(&root, |view, cx| view.resource.get(cx));
        assert_eq!(reloading, ResourceState::Reloading(20));
    }
}
