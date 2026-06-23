use std::{
    cell::{Cell, RefCell},
    future::Future,
    rc::Rc,
};

use gpui::{App, AsyncApp, Context};

use crate::{Effect, Memo, ReactiveContextExt, Resource, ResourceState};

/// A higher-level async query model built on top of [`Resource`].
///
/// `Query` packages the underlying resource together with common derived status
/// memos so app code can consume a stable query model rather than wiring the
/// same loading flags in every surface.
pub struct Query<T, E> {
    resource: Resource<T, E>,
    pending: Memo<bool>,
    loading: Memo<bool>,
    reloading: Memo<bool>,
    ready: Memo<bool>,
    errored: Memo<bool>,
    has_latest: Memo<bool>,
}

/// An entity-scoped query that automatically loads from tracked source state.
///
/// This packages a [`Query`] together with the effect that watches source
/// dependencies and drives `load` / `reload` transitions across the owning
/// GPUI entity's lifetime.
pub struct SourceQuery<T, E> {
    query: Query<T, E>,
    _effect: Rc<Effect>,
    reload_latest: Rc<dyn Fn(&mut App)>,
}

/// Create a pending query model.
pub fn use_query<T: 'static, E: 'static>(cx: &mut App) -> Query<T, E> {
    let resource = Resource::pending(cx);
    Query::from_resource(cx, resource)
}

/// Create a query model that starts with a ready value.
pub fn use_ready_query<T: 'static, E: 'static>(cx: &mut App, value: T) -> Query<T, E> {
    let resource = Resource::ready(cx, value);
    Query::from_resource(cx, resource)
}

/// Create a query model that starts with an error value.
pub fn use_error_query<T: 'static, E: 'static>(cx: &mut App, error: E) -> Query<T, E> {
    let resource = Resource::error(cx, error);
    Query::from_resource(cx, resource)
}

/// Create an entity-scoped query from tracked source state.
///
/// The `source` closure declares and snapshots the tracked dependencies. The
/// first run calls `load`; later source changes call `reload`, preserving the
/// latest ready value during refresh.
pub fn use_query_from_source<T, E, Owner, Source, SourceFn, BuildLoad, Load, Fut>(
    cx: &mut Context<Owner>,
    source: SourceFn,
    build_load: BuildLoad,
) -> SourceQuery<T, E>
where
    Owner: 'static,
    T: 'static,
    E: 'static,
    Source: 'static,
    SourceFn: Fn(&App) -> Source + 'static,
    BuildLoad: Fn(Source) -> Load + 'static,
    Load: FnOnce(AsyncApp) -> Fut + 'static,
    Fut: Future<Output = Result<T, E>> + 'static,
{
    let query = use_query(cx);
    SourceQuery::from_query_and_source(cx, query, source, build_load)
}

impl<T: 'static, E: 'static> Query<T, E> {
    /// Build a query model from an existing resource.
    pub fn from_resource(cx: &mut App, resource: Resource<T, E>) -> Self {
        let pending_resource = resource.clone();
        let loading_resource = resource.clone();
        let reloading_resource = resource.clone();
        let ready_resource = resource.clone();
        let errored_resource = resource.clone();
        let latest_resource = resource.clone();

        Self {
            resource,
            pending: Memo::new(cx, move |cx| pending_resource.is_pending(cx)),
            loading: Memo::new(cx, move |cx| loading_resource.is_loading(cx)),
            reloading: Memo::new(cx, move |cx| reloading_resource.is_reloading(cx)),
            ready: Memo::new(cx, move |cx| ready_resource.is_ready(cx)),
            errored: Memo::new(cx, move |cx| errored_resource.is_error(cx)),
            has_latest: Memo::new(cx, move |cx| latest_resource.has_latest(cx)),
        }
    }

    /// Return the underlying resource.
    pub fn resource(&self) -> &Resource<T, E> {
        &self.resource
    }

    /// Return a memo that is `true` while the query has no value and is loading.
    pub fn pending(&self) -> &Memo<bool> {
        &self.pending
    }

    /// Return a memo that is `true` while the query is loading in any mode.
    pub fn loading(&self) -> &Memo<bool> {
        &self.loading
    }

    /// Return a memo that is `true` while the query reloads with a retained value.
    pub fn reloading(&self) -> &Memo<bool> {
        &self.reloading
    }

    /// Return a memo that is `true` when the query has a ready value.
    pub fn ready(&self) -> &Memo<bool> {
        &self.ready
    }

    /// Return a memo that is `true` when the query is in an error state.
    pub fn errored(&self) -> &Memo<bool> {
        &self.errored
    }

    /// Return a memo that is `true` when the query has a latest usable value.
    pub fn has_latest(&self) -> &Memo<bool> {
        &self.has_latest
    }

    /// Read the underlying state with dependency tracking and without cloning.
    pub fn read<R>(&self, cx: &App, f: impl FnOnce(&ResourceState<T, E>) -> R) -> R {
        self.resource.read(cx, f)
    }

    /// Read the latest available value with dependency tracking.
    pub fn read_latest<R>(&self, cx: &App, f: impl FnOnce(Option<&T>) -> R) -> R {
        self.resource.read_latest(cx, f)
    }

    /// Fold the query state into pending, latest-value, and error branches.
    pub fn fold_latest<R>(
        &self,
        cx: &App,
        pending: impl FnOnce() -> R,
        latest: impl FnOnce(&T, bool) -> R,
        error: impl FnOnce(&E) -> R,
    ) -> R {
        self.resource.fold_latest(cx, pending, latest, error)
    }

    /// Read the current error with dependency tracking.
    pub fn read_error<R>(&self, cx: &App, f: impl FnOnce(Option<&E>) -> R) -> R {
        self.resource.read_error(cx, f)
    }

    /// Return whether the query has no current value and is loading.
    pub fn is_pending(&self, cx: &App) -> bool {
        self.resource.is_pending(cx)
    }

    /// Return whether the query is currently loading.
    pub fn is_loading(&self, cx: &App) -> bool {
        self.resource.is_loading(cx)
    }

    /// Return whether the query is reloading with a retained value.
    pub fn is_reloading(&self, cx: &App) -> bool {
        self.resource.is_reloading(cx)
    }

    /// Return whether the query has a ready value.
    pub fn is_ready(&self, cx: &App) -> bool {
        self.resource.is_ready(cx)
    }

    /// Return whether the query has a latest usable value.
    pub fn has_latest_value(&self, cx: &App) -> bool {
        self.resource.has_latest(cx)
    }

    /// Return whether the query is in an error state.
    pub fn is_error(&self, cx: &App) -> bool {
        self.resource.is_error(cx)
    }

    /// Mark the query as pending.
    pub fn set_pending(&self, cx: &mut App) {
        self.resource.set_pending(cx);
    }

    /// Store a ready value.
    pub fn set_ready(&self, cx: &mut App, value: T) {
        self.resource.set_ready(cx, value);
    }

    /// Store an error value.
    pub fn set_error(&self, cx: &mut App, error: E) {
        self.resource.set_error(cx, error);
    }

    /// Cancel the active load.
    pub fn cancel(&self) {
        self.resource.cancel();
    }

    /// Cancel the active load and restore the retained value when reloading.
    pub fn cancel_to_latest(&self, cx: &mut App) {
        self.resource.cancel_to_latest(cx);
    }
}

impl<T: 'static, E: 'static> SourceQuery<T, E> {
    /// Build a source-driven query model from an existing query.
    pub fn from_query_and_source<Owner, Source, SourceFn, BuildLoad, Load, Fut>(
        cx: &mut Context<Owner>,
        query: Query<T, E>,
        source: SourceFn,
        build_load: BuildLoad,
    ) -> Self
    where
        Owner: 'static,
        Source: 'static,
        SourceFn: Fn(&App) -> Source + 'static,
        BuildLoad: Fn(Source) -> Load + 'static,
        Load: FnOnce(AsyncApp) -> Fut + 'static,
        Fut: Future<Output = Result<T, E>> + 'static,
    {
        let latest_source: Rc<RefCell<Option<Source>>> = Rc::new(RefCell::new(None));
        let latest_source_for_sources = latest_source.clone();
        let initialized = Cell::new(false);
        let query_for_effect = query.clone();
        let source = Rc::new(source);
        let build_load = Rc::new(build_load);
        let source_for_watch = source.clone();
        let source_for_reload = source.clone();
        let build_load_for_effect = build_load.clone();
        let build_load_for_reload = build_load.clone();
        let query_for_reload = query.clone();

        let reload_latest: Rc<dyn Fn(&mut App)> = Rc::new(move |cx| {
            let source = source_for_reload(cx);
            let load = build_load_for_reload(source);
            query_for_reload.reload(cx, load);
        });

        let effect = cx.watch(
            move |cx| {
                *latest_source_for_sources.borrow_mut() = Some(source_for_watch(cx));
            },
            move |cx| {
                let Some(source) = latest_source.borrow_mut().take() else {
                    return;
                };

                let load = build_load_for_effect(source);
                if initialized.replace(true) {
                    query_for_effect.reload(cx, load);
                } else {
                    query_for_effect.load(cx, load);
                }
            },
        );

        Self {
            query,
            _effect: Rc::new(effect),
            reload_latest,
        }
    }

    /// Return the underlying query model.
    pub fn query(&self) -> &Query<T, E> {
        &self.query
    }

    /// Reload the query from the current source snapshot.
    ///
    /// This re-evaluates the source closure on the GPUI app thread and then
    /// uses [`Query::reload`] so the latest ready value remains visible while
    /// refresh work is in flight.
    pub fn reload(&self, cx: &mut App) {
        (self.reload_latest)(cx);
    }

    /// Invalidate the query by immediately refreshing it from the latest source.
    ///
    /// Relay does not currently model a separate stale marker, so invalidation
    /// is an eager reload. This is a semantic helper for mutation follow-up or
    /// command handlers that conceptually "invalidate and refetch."
    pub fn invalidate(&self, cx: &mut App) {
        self.reload(cx);
    }
}

impl<T, E> Query<T, E>
where
    T: 'static,
    E: 'static,
{
    /// Start loading the query from scratch.
    pub fn load<F, Fut>(&self, cx: &mut App, load: F)
    where
        F: FnOnce(AsyncApp) -> Fut + 'static,
        Fut: Future<Output = Result<T, E>> + 'static,
    {
        self.resource.load(cx, load);
    }

    /// Reload the query while retaining the latest ready value.
    pub fn reload<F, Fut>(&self, cx: &mut App, load: F)
    where
        F: FnOnce(AsyncApp) -> Fut + 'static,
        Fut: Future<Output = Result<T, E>> + 'static,
    {
        self.resource.reload(cx, load);
    }
}

impl<T, E> Query<T, E>
where
    T: Clone,
    E: Clone,
{
    /// Clone the current resource state with dependency tracking.
    pub fn get(&self, cx: &App) -> ResourceState<T, E> {
        self.resource.get(cx)
    }
}

impl<T, E> Query<T, E>
where
    T: Clone,
{
    /// Clone the latest available value with dependency tracking.
    pub fn latest(&self, cx: &App) -> Option<T> {
        self.resource.latest(cx)
    }
}

impl<T, E> Query<T, E>
where
    E: Clone,
{
    /// Clone the current error with dependency tracking.
    pub fn error_value(&self, cx: &App) -> Option<E> {
        self.resource.error_value(cx)
    }
}

impl<T, E> Clone for Query<T, E> {
    fn clone(&self) -> Self {
        Self {
            resource: self.resource.clone(),
            pending: self.pending.clone(),
            loading: self.loading.clone(),
            reloading: self.reloading.clone(),
            ready: self.ready.clone(),
            errored: self.errored.clone(),
            has_latest: self.has_latest.clone(),
        }
    }
}

impl<T, E> Clone for SourceQuery<T, E> {
    fn clone(&self) -> Self {
        Self {
            query: self.query.clone(),
            _effect: self._effect.clone(),
            reload_latest: self.reload_latest.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc, time::Duration};

    use gpui::{AppContext, Context, TestApp, TestAppContext};

    use crate::{ResourceState, init};

    use super::*;

    #[test]
    fn query_status_memos_follow_resource_state() {
        let mut app = TestApp::new();
        let query = app.update(|cx| {
            init(cx);
            use_query::<i32, &'static str>(cx)
        });

        app.read(|cx| {
            assert!(query.pending().get(cx));
            assert!(query.loading().get(cx));
            assert!(!query.ready().get(cx));
            assert!(!query.errored().get(cx));
        });

        app.update(|cx| query.set_ready(cx, 7));

        app.read(|cx| {
            assert!(!query.pending().get(cx));
            assert!(!query.loading().get(cx));
            assert!(query.ready().get(cx));
            assert!(query.has_latest().get(cx));
            assert_eq!(query.latest(cx), Some(7));
        });

        app.update(|cx| query.set_error(cx, "failed"));

        app.read(|cx| {
            assert!(query.errored().get(cx));
            assert_eq!(query.error_value(cx), Some("failed"));
        });
    }

    #[gpui::test]
    fn query_reload_uses_reloading_state(cx: &mut TestAppContext) {
        let query = cx.update(|cx| {
            init(cx);
            use_ready_query::<i32, &'static str>(cx, 1)
        });

        cx.update(|cx| {
            query.reload(cx, |cx| async move {
                cx.background_executor()
                    .timer(Duration::from_millis(20))
                    .await;
                Ok(2)
            });
        });

        cx.read(|cx| {
            assert_eq!(query.get(cx), ResourceState::Reloading(1));
            assert!(query.reloading().get(cx));
            assert!(query.loading().get(cx));
        });

        cx.executor().advance_clock(Duration::from_millis(20));
        cx.run_until_parked();

        cx.read(|cx| {
            assert_eq!(query.get(cx), ResourceState::Ready(2));
            assert!(query.ready().get(cx));
            assert!(!query.reloading().get(cx));
        });
    }

    #[gpui::test]
    fn source_query_loads_initially_and_reloads_from_latest_source(cx: &mut TestAppContext) {
        struct QueryHost {
            source: crate::Signal<i32>,
            query: SourceQuery<i32, &'static str>,
        }

        impl QueryHost {
            fn new(cx: &mut Context<Self>, load_inputs: Rc<RefCell<Vec<i32>>>) -> Self {
                init(cx);
                let source = crate::Signal::new(cx, 1);
                let source_for_query = source.clone();
                let query = use_query_from_source(
                    cx,
                    move |cx| source_for_query.get(cx),
                    move |value| {
                        load_inputs.borrow_mut().push(value);
                        move |cx| async move {
                            cx.background_executor()
                                .timer(Duration::from_millis(20))
                                .await;
                            Ok(value * 10)
                        }
                    },
                );

                Self { source, query }
            }
        }

        let load_inputs = Rc::new(RefCell::new(Vec::new()));
        let entity = cx.update({
            let load_inputs = load_inputs.clone();
            move |cx| cx.new(|cx| QueryHost::new(cx, load_inputs))
        });

        assert_eq!(&*load_inputs.borrow(), &[1]);
        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.query.query().get(cx), ResourceState::Pending);
        });

        cx.executor().advance_clock(Duration::from_millis(20));
        cx.run_until_parked();

        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.query.query().get(cx), ResourceState::Ready(10));
        });

        cx.update_entity(&entity, |host, cx| {
            host.source.set(cx, 2);
        });

        assert_eq!(&*load_inputs.borrow(), &[1, 2]);
        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.query.query().get(cx), ResourceState::Reloading(10));
        });

        cx.executor().advance_clock(Duration::from_millis(20));
        cx.run_until_parked();

        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.query.query().get(cx), ResourceState::Ready(20));
        });
    }

    #[gpui::test]
    fn source_query_manual_reload_uses_current_source_snapshot(cx: &mut TestAppContext) {
        struct QueryHost {
            source: crate::Signal<i32>,
            query: SourceQuery<i32, &'static str>,
        }

        impl QueryHost {
            fn new(cx: &mut Context<Self>, load_inputs: Rc<RefCell<Vec<i32>>>) -> Self {
                init(cx);
                let source = crate::Signal::new(cx, 1);
                let source_for_query = source.clone();
                let query = use_query_from_source(
                    cx,
                    move |cx| source_for_query.get(cx),
                    move |value| {
                        load_inputs.borrow_mut().push(value);
                        move |cx| async move {
                            cx.background_executor()
                                .timer(Duration::from_millis(20))
                                .await;
                            Ok(value * 10)
                        }
                    },
                );

                Self { source, query }
            }
        }

        let load_inputs = Rc::new(RefCell::new(Vec::new()));
        let entity = cx.update({
            let load_inputs = load_inputs.clone();
            move |cx| cx.new(|cx| QueryHost::new(cx, load_inputs))
        });

        cx.executor().advance_clock(Duration::from_millis(20));
        cx.run_until_parked();

        cx.update_entity(&entity, |host, cx| {
            host.source.set(cx, 2);
        });

        cx.executor().advance_clock(Duration::from_millis(20));
        cx.run_until_parked();

        cx.update_entity(&entity, |host, cx| {
            host.query.reload(cx);
        });

        assert_eq!(&*load_inputs.borrow(), &[1, 2, 2]);
        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.query.query().get(cx), ResourceState::Reloading(20));
        });

        cx.executor().advance_clock(Duration::from_millis(20));
        cx.run_until_parked();

        cx.read_entity(&entity, |host, cx| {
            assert_eq!(host.query.query().get(cx), ResourceState::Ready(20));
        });
    }
}
