use std::future::Future;

use gpui::{App, AsyncApp};

use crate::{Memo, Resource, ResourceState};

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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use gpui::{TestApp, TestAppContext};

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
}
