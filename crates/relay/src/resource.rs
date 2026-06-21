use std::{cell::RefCell, future::Future, rc::Rc};

use gpui::{App, AsyncApp};

use crate::Signal;

/// Current state of an asynchronous or externally loaded value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceState<T, E> {
    /// The value is loading.
    Pending,
    /// A new value is loading while the previous ready value remains usable.
    Reloading(T),
    /// The value is available.
    Ready(T),
    /// Loading failed.
    Error(E),
}

impl<T, E> ResourceState<T, E> {
    /// Return whether the resource has no current value and is loading.
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }

    /// Return whether the resource is loading while retaining a previous value.
    pub fn is_reloading(&self) -> bool {
        matches!(self, Self::Reloading(_))
    }

    /// Return whether the resource is currently loading.
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Pending | Self::Reloading(_))
    }

    /// Return whether the resource has a ready value.
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }

    /// Return whether the resource has a latest usable value.
    pub fn has_latest(&self) -> bool {
        matches!(self, Self::Reloading(_) | Self::Ready(_))
    }

    /// Return whether the resource has an error.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// Borrow the latest available value, including retained values while reloading.
    pub fn latest(&self) -> Option<&T> {
        match self {
            Self::Reloading(value) | Self::Ready(value) => Some(value),
            Self::Pending | Self::Error(_) => None,
        }
    }

    /// Borrow the current error, if any.
    pub fn error(&self) -> Option<&E> {
        match self {
            Self::Error(error) => Some(error),
            Self::Pending | Self::Reloading(_) | Self::Ready(_) => None,
        }
    }

    /// Fold the state into pending, latest-value, and error branches.
    ///
    /// `Reloading` and `Ready` both call `latest`; the boolean passed to
    /// `latest` is `true` when a refresh is in progress.
    pub fn fold_latest<R>(
        &self,
        pending: impl FnOnce() -> R,
        latest: impl FnOnce(&T, bool) -> R,
        error: impl FnOnce(&E) -> R,
    ) -> R {
        match self {
            Self::Pending => pending(),
            Self::Reloading(value) => latest(value, true),
            Self::Ready(value) => latest(value, false),
            Self::Error(err) => error(err),
        }
    }
}

/// Signal-backed resource state.
///
/// This type is intentionally UI-agnostic. Higher layers decide how to render
/// pending, reloading, ready, and error states.
pub struct Resource<T, E> {
    state: Signal<ResourceState<T, E>>,
    control: Rc<RefCell<ResourceControl>>,
}

#[derive(Default)]
struct ResourceControl {
    generation: u64,
}

impl<T, E> Resource<T, E> {
    /// Create a pending resource.
    pub fn pending(cx: &mut App) -> Self {
        Self {
            state: Signal::new(cx, ResourceState::Pending),
            control: Rc::default(),
        }
    }

    /// Create a ready resource.
    pub fn ready(cx: &mut App, value: T) -> Self {
        Self {
            state: Signal::new(cx, ResourceState::Ready(value)),
            control: Rc::default(),
        }
    }

    /// Create a failed resource.
    pub fn error(cx: &mut App, error: E) -> Self {
        Self {
            state: Signal::new(cx, ResourceState::Error(error)),
            control: Rc::default(),
        }
    }

    /// Return the state signal.
    pub fn signal(&self) -> &Signal<ResourceState<T, E>> {
        &self.state
    }

    /// Read the state with dependency tracking and without cloning.
    pub fn read<R>(&self, cx: &App, f: impl FnOnce(&ResourceState<T, E>) -> R) -> R {
        self.state.read(cx, f)
    }

    /// Read the latest available value with dependency tracking.
    pub fn read_latest<R>(&self, cx: &App, f: impl FnOnce(Option<&T>) -> R) -> R {
        self.read(cx, |state| f(state.latest()))
    }

    /// Read the current error with dependency tracking.
    pub fn read_error<R>(&self, cx: &App, f: impl FnOnce(Option<&E>) -> R) -> R {
        self.read(cx, |state| f(state.error()))
    }

    /// Return whether the resource has no current value and is loading.
    pub fn is_pending(&self, cx: &App) -> bool {
        self.read(cx, ResourceState::is_pending)
    }

    /// Return whether the resource is loading while retaining a previous value.
    pub fn is_reloading(&self, cx: &App) -> bool {
        self.read(cx, ResourceState::is_reloading)
    }

    /// Return whether the resource is currently loading.
    pub fn is_loading(&self, cx: &App) -> bool {
        self.read(cx, ResourceState::is_loading)
    }

    /// Return whether the resource has a ready value.
    pub fn is_ready(&self, cx: &App) -> bool {
        self.read(cx, ResourceState::is_ready)
    }

    /// Return whether the resource has a latest usable value.
    pub fn has_latest(&self, cx: &App) -> bool {
        self.read(cx, ResourceState::has_latest)
    }

    /// Return whether the resource has an error.
    pub fn is_error(&self, cx: &App) -> bool {
        self.read(cx, ResourceState::is_error)
    }

    /// Read the state and fold it into pending, latest-value, and error branches.
    ///
    /// This tracks the resource once and is useful for views that keep rendering
    /// the previous value while a reload is in progress.
    pub fn fold_latest<R>(
        &self,
        cx: &App,
        pending: impl FnOnce() -> R,
        latest: impl FnOnce(&T, bool) -> R,
        error: impl FnOnce(&E) -> R,
    ) -> R {
        self.read(cx, |state| state.fold_latest(pending, latest, error))
    }

    /// Mark the resource as pending.
    pub fn set_pending(&self, cx: &mut App) {
        self.state.update(cx, |state| {
            if matches!(state, ResourceState::Pending) {
                false
            } else {
                *state = ResourceState::Pending;
                true
            }
        });
    }

    /// Store a ready value.
    pub fn set_ready(&self, cx: &mut App, value: T) {
        self.state.update(cx, |state| {
            *state = ResourceState::Ready(value);
            true
        });
    }

    /// Store an error value.
    pub fn set_error(&self, cx: &mut App, error: E) {
        self.state.update(cx, |state| {
            *state = ResourceState::Error(error);
            true
        });
    }

    /// Mark the current load as cancelled.
    ///
    /// Work already handed to GPUI's executor may still finish, but its
    /// completion is ignored. The visible state is left unchanged.
    pub fn cancel(&self) {
        let mut control = self.control.borrow_mut();
        control.generation = control.generation.wrapping_add(1);
    }

    /// Cancel the current load and restore a retained value to ready state.
    ///
    /// If the resource is not reloading, this behaves like [`Resource::cancel`]
    /// and leaves the visible state unchanged.
    pub fn cancel_to_latest(&self, cx: &mut App) {
        self.cancel();
        self.state.update(cx, |state| {
            let current = std::mem::replace(state, ResourceState::Pending);
            match current {
                ResourceState::Reloading(value) => {
                    *state = ResourceState::Ready(value);
                    true
                }
                current => {
                    *state = current;
                    false
                }
            }
        });
    }
}

impl<T, E> Resource<T, E>
where
    T: 'static,
    E: 'static,
{
    /// Start loading the resource on GPUI's foreground executor.
    ///
    /// The resource is reset to [`ResourceState::Pending`] immediately. When
    /// the future resolves, the latest load commits [`ResourceState::Ready`] or
    /// [`ResourceState::Error`] on the GPUI app thread. Starting another load
    /// prevents stale completions from committing. Use [`Resource::reload`] to
    /// retain an existing value while loading.
    pub fn load<F, Fut>(&self, cx: &mut App, load: F)
    where
        F: FnOnce(AsyncApp) -> Fut + 'static,
        Fut: Future<Output = Result<T, E>> + 'static,
    {
        self.start_load(cx, LoadMode::Reset, load);
    }

    /// Restart loading the resource.
    ///
    /// If the resource currently has a ready or retained value, it enters
    /// [`ResourceState::Reloading`] so render code can keep showing the latest
    /// value while indicating refresh progress.
    pub fn reload<F, Fut>(&self, cx: &mut App, load: F)
    where
        F: FnOnce(AsyncApp) -> Fut + 'static,
        Fut: Future<Output = Result<T, E>> + 'static,
    {
        self.start_load(cx, LoadMode::RetainLatest, load);
    }

    fn start_load<F, Fut>(&self, cx: &mut App, mode: LoadMode, load: F)
    where
        F: FnOnce(AsyncApp) -> Fut + 'static,
        Fut: Future<Output = Result<T, E>> + 'static,
    {
        let generation = self.begin_load(cx, mode);
        let state = self.state.clone();
        let control = Rc::downgrade(&self.control);

        cx.spawn(async move |cx| {
            let result = load(cx.clone()).await;
            cx.update(move |cx| {
                let Some(control) = control.upgrade() else {
                    return;
                };
                if control.borrow().generation != generation {
                    return;
                }
                match result {
                    Ok(value) => {
                        state.update(cx, |state| {
                            *state = ResourceState::Ready(value);
                            true
                        });
                    }
                    Err(error) => {
                        state.update(cx, |state| {
                            *state = ResourceState::Error(error);
                            true
                        });
                    }
                }
            });
        })
        .detach();
    }

    fn begin_load(&self, cx: &mut App, mode: LoadMode) -> u64 {
        let mut control = self.control.borrow_mut();
        control.generation = control.generation.wrapping_add(1);
        let generation = control.generation;
        drop(control);

        self.state.update(cx, |state| match mode {
            LoadMode::Reset => {
                if matches!(state, ResourceState::Pending) {
                    false
                } else {
                    *state = ResourceState::Pending;
                    true
                }
            }
            LoadMode::RetainLatest => {
                let current = std::mem::replace(state, ResourceState::Pending);
                match current {
                    ResourceState::Ready(value) => {
                        *state = ResourceState::Reloading(value);
                        true
                    }
                    ResourceState::Reloading(value) => {
                        *state = ResourceState::Reloading(value);
                        false
                    }
                    ResourceState::Pending => false,
                    ResourceState::Error(_) => true,
                }
            }
        });

        generation
    }
}

#[derive(Clone, Copy)]
enum LoadMode {
    Reset,
    RetainLatest,
}

impl<T, E> Resource<T, E>
where
    T: Clone,
    E: Clone,
{
    /// Clone the current resource state with dependency tracking.
    pub fn get(&self, cx: &App) -> ResourceState<T, E> {
        self.state.get(cx)
    }
}

impl<T, E> Resource<T, E>
where
    T: Clone,
{
    /// Clone the latest available value with dependency tracking.
    pub fn latest(&self, cx: &App) -> Option<T> {
        self.read_latest(cx, |latest| latest.cloned())
    }
}

impl<T, E> Resource<T, E>
where
    E: Clone,
{
    /// Clone the current error with dependency tracking.
    pub fn error_value(&self, cx: &App) -> Option<E> {
        self.read_error(cx, |error| error.cloned())
    }
}

impl<T, E> Clone for Resource<T, E> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            control: self.control.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use gpui::{TestApp, TestAppContext};

    use crate::init;

    use super::*;

    #[test]
    fn resource_transitions_from_pending_to_ready() {
        let mut app = TestApp::new();
        let resource = app.update(|cx| {
            init(cx);
            Resource::<i32, &'static str>::pending(cx)
        });

        app.update(|cx| {
            resource.set_ready(cx, 7);
        });

        app.read(|cx| {
            assert_eq!(resource.get(cx), ResourceState::Ready(7));
        });
    }

    #[test]
    fn resource_stores_error_state() {
        let mut app = TestApp::new();
        let resource = app.update(|cx| {
            init(cx);
            Resource::<i32, &'static str>::ready(cx, 7)
        });

        app.update(|cx| {
            resource.set_error(cx, "failed");
        });

        app.read(|cx| {
            assert_eq!(resource.get(cx), ResourceState::Error("failed"));
        });
    }

    #[test]
    fn resource_state_latest_borrows_ready_and_reloading_values() {
        let ready = ResourceState::<_, &'static str>::Ready(7);
        assert_eq!(ready.latest(), Some(&7));

        let reloading = ResourceState::<_, &'static str>::Reloading(8);
        assert_eq!(reloading.latest(), Some(&8));
    }

    #[test]
    fn resource_state_has_latest_for_ready_and_reloading() {
        let ready = ResourceState::<_, &'static str>::Ready(7);
        assert!(ready.has_latest());

        let reloading = ResourceState::<_, &'static str>::Reloading(8);
        assert!(reloading.has_latest());

        let pending = ResourceState::<i32, &'static str>::Pending;
        assert!(!pending.has_latest());
    }

    #[test]
    fn resource_state_fold_latest_collapses_ready_and_reloading() {
        let ready = ResourceState::<_, &'static str>::Ready(7);
        let reloading = ResourceState::<_, &'static str>::Reloading(8);

        let ready_result = ready.fold_latest(
            || "pending".to_string(),
            |value, loading| format!("{value}:{loading}"),
            |error| error.to_string(),
        );
        let reloading_result = reloading.fold_latest(
            || "pending".to_string(),
            |value, loading| format!("{value}:{loading}"),
            |error| error.to_string(),
        );

        assert_eq!(ready_result, "7:false");
        assert_eq!(reloading_result, "8:true");
    }

    #[test]
    fn resource_fold_latest_reads_ready_value() {
        let mut app = TestApp::new();
        let resource = app.update(|cx| {
            init(cx);
            Resource::<i32, &'static str>::ready(cx, 7)
        });

        let result = app.read(|cx| {
            resource.fold_latest(
                cx,
                || 0,
                |value, loading| value + if loading { 1 } else { 0 },
                |_| -1,
            )
        });

        assert_eq!(result, 7);
    }

    #[test]
    fn resource_status_queries_read_current_state() {
        let mut app = TestApp::new();
        let resource = app.update(|cx| {
            init(cx);
            Resource::<i32, &'static str>::ready(cx, 7)
        });

        app.read(|cx| {
            assert!(resource.is_ready(cx));
            assert!(resource.has_latest(cx));
            assert!(!resource.is_loading(cx));
        });

        app.update(|cx| {
            resource.set_pending(cx);
        });

        app.read(|cx| {
            assert!(resource.is_pending(cx));
            assert!(resource.is_loading(cx));
            assert!(!resource.has_latest(cx));
        });
    }

    #[test]
    fn resource_read_error_borrows_non_clone_error() {
        struct NonCloneError(&'static str);

        let mut app = TestApp::new();
        let resource = app.update(|cx| {
            init(cx);
            Resource::<i32, NonCloneError>::error(cx, NonCloneError("failed"))
        });

        let label = app.read(|cx| resource.read_error(cx, |error| error.map(|error| error.0)));

        assert_eq!(label, Some("failed"));
    }

    #[test]
    fn resource_error_value_clones_current_error() {
        let mut app = TestApp::new();
        let resource = app.update(|cx| {
            init(cx);
            Resource::<i32, String>::error(cx, "failed".to_string())
        });

        let error = app.read(|cx| resource.error_value(cx));

        assert_eq!(error, Some("failed".to_string()));
    }

    #[gpui::test]
    fn resource_load_resolves_to_ready(cx: &mut TestAppContext) {
        let resource = cx.update(|cx| {
            init(cx);
            Resource::<i32, &'static str>::pending(cx)
        });

        cx.update(|cx| {
            resource.load(cx, |_| async move { Ok(7) });
        });

        cx.run_until_parked();

        cx.read(|cx| {
            assert_eq!(resource.get(cx), ResourceState::Ready(7));
        });
    }

    #[gpui::test]
    fn resource_load_resets_previous_value_while_pending(cx: &mut TestAppContext) {
        let resource = cx.update(|cx| {
            init(cx);
            Resource::<i32, &'static str>::ready(cx, 1)
        });

        cx.update(|cx| {
            resource.load(cx, |_| async move { Ok(2) });
        });

        cx.read(|cx| {
            assert_eq!(resource.get(cx), ResourceState::Pending);
            assert_eq!(resource.latest(cx), None);
        });

        cx.run_until_parked();

        cx.read(|cx| {
            assert_eq!(resource.get(cx), ResourceState::Ready(2));
        });
    }

    #[gpui::test]
    fn resource_reload_retains_latest_value_while_loading(cx: &mut TestAppContext) {
        let resource = cx.update(|cx| {
            init(cx);
            Resource::<i32, &'static str>::ready(cx, 1)
        });

        cx.update(|cx| {
            resource.reload(cx, |_| async move { Ok(2) });
        });

        cx.read(|cx| {
            assert_eq!(resource.get(cx), ResourceState::Reloading(1));
            assert_eq!(resource.latest(cx), Some(1));
        });

        cx.run_until_parked();

        cx.read(|cx| {
            assert_eq!(resource.get(cx), ResourceState::Ready(2));
        });
    }

    #[gpui::test]
    fn resource_reload_from_error_enters_pending_without_latest(cx: &mut TestAppContext) {
        let resource = cx.update(|cx| {
            init(cx);
            Resource::<i32, &'static str>::error(cx, "failed")
        });

        cx.update(|cx| {
            resource.reload(cx, |cx| async move {
                cx.background_executor()
                    .timer(Duration::from_millis(20))
                    .await;
                Ok(2)
            });
        });

        cx.read(|cx| {
            assert_eq!(resource.get(cx), ResourceState::Pending);
            assert_eq!(resource.latest(cx), None);
        });
    }

    #[gpui::test]
    fn resource_cancel_to_latest_restores_retained_value(cx: &mut TestAppContext) {
        let resource = cx.update(|cx| {
            init(cx);
            Resource::<i32, &'static str>::ready(cx, 1)
        });

        cx.update(|cx| {
            resource.reload(cx, |cx| async move {
                cx.background_executor()
                    .timer(Duration::from_millis(20))
                    .await;
                Ok(2)
            });
            resource.cancel_to_latest(cx);
        });

        cx.run_until_parked();

        cx.read(|cx| {
            assert_eq!(resource.get(cx), ResourceState::Ready(1));
        });
    }

    #[gpui::test]
    fn resource_load_resolves_to_error(cx: &mut TestAppContext) {
        let resource = cx.update(|cx| {
            init(cx);
            Resource::<i32, &'static str>::pending(cx)
        });

        cx.update(|cx| {
            resource.load(cx, |_| async move { Err("failed") });
        });

        cx.run_until_parked();

        cx.read(|cx| {
            assert_eq!(resource.get(cx), ResourceState::Error("failed"));
        });
    }

    #[gpui::test]
    fn resource_ignores_stale_load_completion(cx: &mut TestAppContext) {
        let resource = cx.update(|cx| {
            init(cx);
            Resource::<i32, &'static str>::pending(cx)
        });

        cx.update(|cx| {
            resource.load(cx, |cx| async move {
                cx.background_executor()
                    .timer(Duration::from_millis(20))
                    .await;
                Ok(1)
            });
            resource.load(cx, |_| async move { Ok(2) });
        });

        cx.run_until_parked();

        cx.read(|cx| {
            assert_eq!(resource.get(cx), ResourceState::Ready(2));
        });
    }

    #[gpui::test]
    fn resource_cancel_ignores_load_completion(cx: &mut TestAppContext) {
        let resource = cx.update(|cx| {
            init(cx);
            Resource::<i32, &'static str>::ready(cx, 1)
        });

        cx.update(|cx| {
            resource.load(cx, |_| async move { Ok(2) });
            resource.cancel();
        });

        cx.run_until_parked();

        cx.read(|cx| {
            assert_eq!(resource.get(cx), ResourceState::Pending);
        });
    }
}
