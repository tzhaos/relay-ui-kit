use std::{cell::RefCell, future::Future, rc::Rc};

use gpui::{App, AsyncApp};

use crate::Signal;

/// Current state of an asynchronous or externally loaded value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceState<T, E> {
    /// The value is loading.
    Pending,
    /// The value is available.
    Ready(T),
    /// Loading failed.
    Error(E),
}

/// Signal-backed resource state.
///
/// This type is intentionally UI-agnostic. Higher layers decide how to render
/// pending, ready, and error states.
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
}

impl<T, E> Resource<T, E>
where
    T: 'static,
    E: 'static,
{
    /// Start loading the resource on GPUI's foreground executor.
    ///
    /// The resource is set to [`ResourceState::Pending`] immediately. When the
    /// future resolves, the latest load commits [`ResourceState::Ready`] or
    /// [`ResourceState::Error`] on the GPUI app thread. Starting another load
    /// prevents stale completions from committing.
    pub fn load<F, Fut>(&self, cx: &mut App, load: F)
    where
        F: FnOnce(AsyncApp) -> Fut + 'static,
        Fut: Future<Output = Result<T, E>> + 'static,
    {
        self.reload(cx, load);
    }

    /// Restart loading the resource.
    ///
    /// This is an alias for [`Resource::load`] that reads better at call sites
    /// where the resource may already contain a value.
    pub fn reload<F, Fut>(&self, cx: &mut App, load: F)
    where
        F: FnOnce(AsyncApp) -> Fut + 'static,
        Fut: Future<Output = Result<T, E>> + 'static,
    {
        let generation = self.begin_load(cx);
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

    fn begin_load(&self, cx: &mut App) -> u64 {
        self.set_pending(cx);
        let mut control = self.control.borrow_mut();
        control.generation = control.generation.wrapping_add(1);
        control.generation
    }
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
