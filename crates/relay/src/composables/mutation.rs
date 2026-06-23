use std::{cell::RefCell, future::Future, rc::Rc};

use gpui::{App, AsyncApp};

use crate::{Memo, Signal, batch};

type OptimisticRollback = Box<dyn FnOnce(&mut App)>;

/// Current state of an asynchronous mutation.
///
/// Mutations are action-driven rather than source-driven. Relay retains the
/// latest successful result across retries so UI code can keep showing the last
/// committed outcome while a new mutation is pending or after a failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutationState<T, E> {
    /// No mutation has run yet, or the state has been reset.
    Idle,
    /// A mutation is currently running.
    Pending {
        /// The latest successful result, if any.
        last_success: Option<T>,
    },
    /// The latest mutation completed successfully.
    Succeeded(T),
    /// The latest mutation failed.
    Failed {
        /// The current error.
        error: E,
        /// The latest successful result retained across retries.
        last_success: Option<T>,
    },
}

impl<T, E> MutationState<T, E> {
    /// Return whether the mutation is idle.
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    /// Return whether the mutation is currently pending.
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending { .. })
    }

    /// Return whether the latest mutation succeeded.
    pub fn is_succeeded(&self) -> bool {
        matches!(self, Self::Succeeded(_))
    }

    /// Return whether the latest mutation failed.
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }

    /// Return whether the mutation retains a latest successful result.
    pub fn has_last_success(&self) -> bool {
        self.last_success().is_some()
    }

    /// Borrow the latest successful result, including retained values.
    pub fn last_success(&self) -> Option<&T> {
        match self {
            Self::Idle => None,
            Self::Pending { last_success } | Self::Failed { last_success, .. } => {
                last_success.as_ref()
            }
            Self::Succeeded(value) => Some(value),
        }
    }

    /// Borrow the current error, if any.
    pub fn error(&self) -> Option<&E> {
        match self {
            Self::Failed { error, .. } => Some(error),
            Self::Idle | Self::Pending { .. } | Self::Succeeded(_) => None,
        }
    }

    /// Fold the state into idle, pending, success, and failure branches.
    pub fn fold<R>(
        &self,
        idle: impl FnOnce() -> R,
        pending: impl FnOnce(Option<&T>) -> R,
        success: impl FnOnce(&T) -> R,
        failed: impl FnOnce(&E, Option<&T>) -> R,
    ) -> R {
        match self {
            Self::Idle => idle(),
            Self::Pending { last_success } => pending(last_success.as_ref()),
            Self::Succeeded(value) => success(value),
            Self::Failed {
                error,
                last_success,
            } => failed(error, last_success.as_ref()),
        }
    }

    fn into_last_success(self) -> Option<T> {
        match self {
            Self::Idle => None,
            Self::Pending { last_success } | Self::Failed { last_success, .. } => last_success,
            Self::Succeeded(value) => Some(value),
        }
    }
}

/// A higher-level async mutation model.
pub struct Mutation<T, E> {
    state: Signal<MutationState<T, E>>,
    control: Rc<RefCell<MutationControl>>,
    idle: Memo<bool>,
    pending: Memo<bool>,
    succeeded: Memo<bool>,
    failed: Memo<bool>,
    has_last_success: Memo<bool>,
}

#[derive(Default)]
struct MutationControl {
    generation: u64,
    optimistic_rollback: Option<OptimisticRollback>,
}

/// Create a mutation model in the idle state.
pub fn use_mutation<T: 'static, E: 'static>(cx: &mut App) -> Mutation<T, E> {
    Mutation::new(cx)
}

impl<T: 'static, E: 'static> Mutation<T, E> {
    /// Create an idle mutation model.
    pub fn new(cx: &mut App) -> Self {
        let state = Signal::new(cx, MutationState::Idle);
        Self::from_state(cx, state)
    }

    /// Build a mutation model from an existing state signal.
    pub fn from_state(cx: &mut App, state: Signal<MutationState<T, E>>) -> Self {
        let idle_state = state.clone();
        let pending_state = state.clone();
        let succeeded_state = state.clone();
        let failed_state = state.clone();
        let latest_state = state.clone();

        Self {
            state,
            control: Rc::default(),
            idle: Memo::new(cx, move |cx| idle_state.read(cx, MutationState::is_idle)),
            pending: Memo::new(cx, move |cx| {
                pending_state.read(cx, MutationState::is_pending)
            }),
            succeeded: Memo::new(cx, move |cx| {
                succeeded_state.read(cx, MutationState::is_succeeded)
            }),
            failed: Memo::new(cx, move |cx| {
                failed_state.read(cx, MutationState::is_failed)
            }),
            has_last_success: Memo::new(cx, move |cx| {
                latest_state.read(cx, MutationState::has_last_success)
            }),
        }
    }

    /// Return the underlying state signal.
    pub fn signal(&self) -> &Signal<MutationState<T, E>> {
        &self.state
    }

    /// Return a memo that is `true` when the mutation is idle.
    pub fn idle(&self) -> &Memo<bool> {
        &self.idle
    }

    /// Return a memo that is `true` while the mutation is running.
    pub fn pending(&self) -> &Memo<bool> {
        &self.pending
    }

    /// Return a memo that is `true` when the latest mutation succeeded.
    pub fn succeeded(&self) -> &Memo<bool> {
        &self.succeeded
    }

    /// Return a memo that is `true` when the latest mutation failed.
    pub fn failed(&self) -> &Memo<bool> {
        &self.failed
    }

    /// Return a memo that is `true` when the mutation retains a successful result.
    pub fn has_last_success(&self) -> &Memo<bool> {
        &self.has_last_success
    }

    /// Read the current state with dependency tracking and without cloning.
    pub fn read<R>(&self, cx: &App, f: impl FnOnce(&MutationState<T, E>) -> R) -> R {
        self.state.read(cx, f)
    }

    /// Read the latest successful result with dependency tracking.
    pub fn read_last_success<R>(&self, cx: &App, f: impl FnOnce(Option<&T>) -> R) -> R {
        self.read(cx, |state| f(state.last_success()))
    }

    /// Read the current error with dependency tracking.
    pub fn read_error<R>(&self, cx: &App, f: impl FnOnce(Option<&E>) -> R) -> R {
        self.read(cx, |state| f(state.error()))
    }

    /// Fold the current state into idle, pending, success, and failure branches.
    pub fn fold<R>(
        &self,
        cx: &App,
        idle: impl FnOnce() -> R,
        pending: impl FnOnce(Option<&T>) -> R,
        success: impl FnOnce(&T) -> R,
        failed: impl FnOnce(&E, Option<&T>) -> R,
    ) -> R {
        self.read(cx, |state| state.fold(idle, pending, success, failed))
    }

    /// Return whether the mutation is idle.
    pub fn is_idle(&self, cx: &App) -> bool {
        self.read(cx, MutationState::is_idle)
    }

    /// Return whether the mutation is currently pending.
    pub fn is_pending(&self, cx: &App) -> bool {
        self.read(cx, MutationState::is_pending)
    }

    /// Return whether the latest mutation succeeded.
    pub fn is_succeeded(&self, cx: &App) -> bool {
        self.read(cx, MutationState::is_succeeded)
    }

    /// Return whether the latest mutation failed.
    pub fn is_failed(&self, cx: &App) -> bool {
        self.read(cx, MutationState::is_failed)
    }

    /// Reset the mutation back to the idle state.
    pub fn reset(&self, cx: &mut App) {
        batch(cx, |cx| {
            self.invalidate_active_run();
            self.rollback_active_optimistic(cx);
            self.state.update(cx, |state| {
                *state = MutationState::Idle;
                true
            });
        });
    }

    /// Store a successful mutation result.
    pub fn set_succeeded(&self, cx: &mut App, value: T) {
        batch(cx, |cx| {
            self.clear_active_optimistic();
            self.state.update(cx, |state| {
                *state = MutationState::Succeeded(value);
                true
            });
        });
    }

    /// Store a failed mutation result.
    pub fn set_failed(&self, cx: &mut App, error: E) {
        batch(cx, |cx| {
            self.rollback_active_optimistic(cx);
            self.state.update(cx, |state| {
                let current = std::mem::replace(state, MutationState::Idle);
                *state = MutationState::Failed {
                    error,
                    last_success: current.into_last_success(),
                };
                true
            });
        });
    }

    /// Cancel the active mutation and restore the latest stable state.
    pub fn cancel(&self, cx: &mut App) {
        batch(cx, |cx| {
            self.invalidate_active_run();
            self.rollback_active_optimistic(cx);

            self.state.update(cx, |state| {
                let current = std::mem::replace(state, MutationState::Idle);
                match current {
                    MutationState::Pending { last_success } => {
                        *state = match last_success {
                            Some(value) => MutationState::Succeeded(value),
                            None => MutationState::Idle,
                        };
                        true
                    }
                    current => {
                        *state = current;
                        false
                    }
                }
            });
        });
    }

    /// Run the mutation on GPUI's foreground executor.
    ///
    /// Starting a new mutation ignores stale completion from previous runs.
    pub fn run<F, Fut>(&self, cx: &mut App, run: F)
    where
        F: FnOnce(AsyncApp) -> Fut + 'static,
        Fut: Future<Output = Result<T, E>> + 'static,
    {
        self.run_with_followup(cx, run, |_cx, _mutation| {});
    }

    /// Run the mutation with an immediate optimistic update.
    ///
    /// `optimistic` runs synchronously on the GPUI app thread before the async
    /// work is spawned. It can update external state and must return a rollback
    /// closure that restores the previous stable state. Relay runs that
    /// rollback if the mutation fails, is cancelled, is reset, or is superseded
    /// by a newer run.
    pub fn run_optimistic<F, Fut, Optimistic, Rollback>(
        &self,
        cx: &mut App,
        optimistic: Optimistic,
        run: F,
    ) where
        F: FnOnce(AsyncApp) -> Fut + 'static,
        Fut: Future<Output = Result<T, E>> + 'static,
        Optimistic: FnOnce(&mut App) -> Rollback + 'static,
        Rollback: FnOnce(&mut App) + 'static,
    {
        self.run_optimistic_with_followup(cx, optimistic, run, |_cx, _mutation| {});
    }

    /// Run the mutation and invoke a follow-up callback after the latest completion.
    ///
    /// The follow-up runs on the GPUI app thread after relay commits the latest
    /// result to the mutation state, making it suitable for query invalidation,
    /// form reset, or signal synchronization.
    pub fn run_with_followup<F, Fut, Followup>(&self, cx: &mut App, run: F, followup: Followup)
    where
        F: FnOnce(AsyncApp) -> Fut + 'static,
        Fut: Future<Output = Result<T, E>> + 'static,
        Followup: FnOnce(&mut App, &Mutation<T, E>) + 'static,
    {
        let generation = batch(cx, |cx| self.begin_run(cx));
        let control = Rc::downgrade(&self.control);
        let mutation = self.clone();

        cx.spawn(async move |cx| {
            let result = run(cx.clone()).await;
            cx.update(move |cx| {
                let Some(control) = control.upgrade() else {
                    return;
                };
                if control.borrow().generation != generation {
                    return;
                }

                mutation.commit_result(cx, result);
                followup(cx, &mutation);
            });
        })
        .detach();
    }

    /// Run the mutation with an optimistic update and a latest-only follow-up.
    ///
    /// The optimistic closure runs immediately and returns a rollback closure.
    /// Relay keeps that rollback alive only for the active pending run. On
    /// success, the rollback is discarded. On failure, cancel, reset, or when
    /// a newer run supersedes this one, relay executes the rollback on the
    /// GPUI app thread before the surface observes the reverted state.
    ///
    /// The follow-up runs only for the latest completion, after relay commits
    /// the mutation result and after any rollback required by an error has
    /// already run.
    pub fn run_optimistic_with_followup<F, Fut, Optimistic, Rollback, Followup>(
        &self,
        cx: &mut App,
        optimistic: Optimistic,
        run: F,
        followup: Followup,
    ) where
        F: FnOnce(AsyncApp) -> Fut + 'static,
        Fut: Future<Output = Result<T, E>> + 'static,
        Optimistic: FnOnce(&mut App) -> Rollback + 'static,
        Rollback: FnOnce(&mut App) + 'static,
        Followup: FnOnce(&mut App, &Mutation<T, E>) + 'static,
    {
        let generation = batch(cx, |cx| {
            let generation = self.begin_run(cx);
            let rollback = optimistic(cx);
            self.install_active_optimistic(Box::new(rollback));
            generation
        });
        let control = Rc::downgrade(&self.control);
        let mutation = self.clone();

        cx.spawn(async move |cx| {
            let result = run(cx.clone()).await;
            cx.update(move |cx| {
                let Some(control) = control.upgrade() else {
                    return;
                };
                if control.borrow().generation != generation {
                    return;
                }

                mutation.commit_result(cx, result);
                followup(cx, &mutation);
            });
        })
        .detach();
    }

    fn begin_run(&self, cx: &mut App) -> u64 {
        let generation = self.invalidate_active_run();
        self.rollback_active_optimistic(cx);

        self.state.update(cx, |state| {
            let current = std::mem::replace(state, MutationState::Idle);
            *state = MutationState::Pending {
                last_success: current.into_last_success(),
            };
            true
        });

        generation
    }

    fn commit_result(&self, cx: &mut App, result: Result<T, E>) {
        let rollback_on_error = result.is_err();
        batch(cx, |cx| {
            if rollback_on_error {
                self.rollback_active_optimistic(cx);
            } else {
                self.clear_active_optimistic();
            }

            self.state.update(cx, |state| {
                let current = std::mem::replace(state, MutationState::Idle);
                *state = match result {
                    Ok(value) => MutationState::Succeeded(value),
                    Err(error) => MutationState::Failed {
                        error,
                        last_success: current.into_last_success(),
                    },
                };
                true
            });
        });
    }

    fn invalidate_active_run(&self) -> u64 {
        let mut control = self.control.borrow_mut();
        control.generation = control.generation.wrapping_add(1);
        control.generation
    }

    fn install_active_optimistic(&self, rollback: OptimisticRollback) {
        self.control.borrow_mut().optimistic_rollback = Some(rollback);
    }

    fn clear_active_optimistic(&self) {
        self.control.borrow_mut().optimistic_rollback = None;
    }

    fn rollback_active_optimistic(&self, cx: &mut App) {
        let rollback = self.control.borrow_mut().optimistic_rollback.take();
        if let Some(rollback) = rollback {
            rollback(cx);
        }
    }
}

impl<T, E> Mutation<T, E>
where
    T: Clone + 'static,
    E: Clone + 'static,
{
    /// Clone the current mutation state with dependency tracking.
    pub fn get(&self, cx: &App) -> MutationState<T, E> {
        self.state.get(cx)
    }
}

impl<T, E> Mutation<T, E>
where
    T: Clone + 'static,
    E: 'static,
{
    /// Clone the latest successful result with dependency tracking.
    pub fn last_success_value(&self, cx: &App) -> Option<T> {
        self.read_last_success(cx, |value| value.cloned())
    }
}

impl<T, E> Mutation<T, E>
where
    T: 'static,
    E: Clone + 'static,
{
    /// Clone the current error with dependency tracking.
    pub fn error_value(&self, cx: &App) -> Option<E> {
        self.read_error(cx, |error| error.cloned())
    }
}

impl<T, E> Clone for Mutation<T, E> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            control: self.control.clone(),
            idle: self.idle.clone(),
            pending: self.pending.clone(),
            succeeded: self.succeeded.clone(),
            failed: self.failed.clone(),
            has_last_success: self.has_last_success.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
        time::Duration,
    };

    use gpui::{TestApp, TestAppContext};

    use crate::init;

    use super::*;

    #[test]
    fn mutation_status_memos_follow_state_transitions() {
        let mut app = TestApp::new();
        let mutation = app.update(|cx| {
            init(cx);
            use_mutation::<i32, &'static str>(cx)
        });

        app.read(|cx| {
            assert!(mutation.idle().get(cx));
            assert!(!mutation.pending().get(cx));
            assert!(!mutation.succeeded().get(cx));
            assert!(!mutation.failed().get(cx));
        });

        app.update(|cx| mutation.set_succeeded(cx, 7));

        app.read(|cx| {
            assert!(mutation.succeeded().get(cx));
            assert!(mutation.has_last_success().get(cx));
            assert_eq!(mutation.last_success_value(cx), Some(7));
        });

        app.update(|cx| mutation.set_failed(cx, "failed"));

        app.read(|cx| {
            assert!(mutation.failed().get(cx));
            assert_eq!(mutation.error_value(cx), Some("failed"));
            assert_eq!(mutation.last_success_value(cx), Some(7));
        });
    }

    #[gpui::test]
    fn mutation_failure_retains_last_success(cx: &mut TestAppContext) {
        let mutation = cx.update(|cx| {
            init(cx);
            use_mutation::<i32, &'static str>(cx)
        });

        cx.update(|cx| {
            mutation.set_succeeded(cx, 10);
            mutation.run(cx, |cx| async move {
                cx.background_executor()
                    .timer(Duration::from_millis(20))
                    .await;
                Err("failed")
            });
        });

        cx.read(|cx| {
            assert_eq!(
                mutation.get(cx),
                MutationState::Pending {
                    last_success: Some(10),
                }
            );
        });

        cx.executor().advance_clock(Duration::from_millis(20));
        cx.run_until_parked();

        cx.read(|cx| {
            assert_eq!(
                mutation.get(cx),
                MutationState::Failed {
                    error: "failed",
                    last_success: Some(10),
                }
            );
        });
    }

    #[gpui::test]
    fn mutation_cancel_restores_latest_stable_state(cx: &mut TestAppContext) {
        let mutation = cx.update(|cx| {
            init(cx);
            use_mutation::<i32, &'static str>(cx)
        });

        cx.update(|cx| {
            mutation.set_succeeded(cx, 5);
            mutation.run(cx, |cx| async move {
                cx.background_executor()
                    .timer(Duration::from_millis(20))
                    .await;
                Ok(9)
            });
            mutation.cancel(cx);
        });

        cx.read(|cx| {
            assert_eq!(mutation.get(cx), MutationState::Succeeded(5));
        });
    }

    #[gpui::test]
    fn mutation_ignores_stale_completion(cx: &mut TestAppContext) {
        let mutation = cx.update(|cx| {
            init(cx);
            use_mutation::<i32, &'static str>(cx)
        });

        cx.update(|cx| {
            mutation.run(cx, |cx| async move {
                cx.background_executor()
                    .timer(Duration::from_millis(20))
                    .await;
                Ok(1)
            });
            mutation.run(cx, |_| async move { Ok(2) });
        });

        cx.executor().advance_clock(Duration::from_millis(20));
        cx.run_until_parked();

        cx.read(|cx| {
            assert_eq!(mutation.get(cx), MutationState::Succeeded(2));
        });
    }

    #[gpui::test]
    fn mutation_followup_runs_after_latest_completion(cx: &mut TestAppContext) {
        let mutation = cx.update(|cx| {
            init(cx);
            use_mutation::<i32, &'static str>(cx)
        });
        let followups = Rc::new(Cell::new(0));
        let seen_values = Rc::new(RefCell::new(Vec::new()));
        let mutation_for_run = mutation.clone();

        cx.update({
            let followups = followups.clone();
            let seen_values = seen_values.clone();
            move |cx| {
                mutation_for_run.run_with_followup(
                    cx,
                    |_| async move { Ok(7) },
                    move |cx, mutation| {
                        followups.set(followups.get() + 1);
                        seen_values
                            .borrow_mut()
                            .push(mutation.last_success_value(cx).unwrap_or_default());
                    },
                );
            }
        });

        cx.run_until_parked();

        cx.read(|cx| {
            assert_eq!(mutation.get(cx), MutationState::Succeeded(7));
        });
        assert_eq!(followups.get(), 1);
        assert_eq!(&*seen_values.borrow(), &[7]);
    }

    #[gpui::test]
    fn optimistic_mutation_failure_rolls_back_external_state(cx: &mut TestAppContext) {
        let (mutation, server_value) = cx.update(|cx| {
            init(cx);
            (
                use_mutation::<String, &'static str>(cx),
                Signal::new(cx, "stable".to_string()),
            )
        });

        cx.update({
            let mutation_for_run = mutation.clone();
            let server_value = server_value.clone();
            move |cx| {
                mutation_for_run.run_optimistic(
                    cx,
                    move |cx| {
                        let previous = server_value.get(cx);
                        server_value.set(cx, "optimistic".to_string());
                        move |cx| {
                            server_value.set(cx, previous);
                        }
                    },
                    |cx| async move {
                        cx.background_executor()
                            .timer(Duration::from_millis(20))
                            .await;
                        Err("failed")
                    },
                );
            }
        });

        cx.read(|cx| {
            assert_eq!(server_value.get(cx), "optimistic".to_string());
            assert!(mutation.is_pending(cx));
        });

        cx.executor().advance_clock(Duration::from_millis(20));
        cx.run_until_parked();

        cx.read(|cx| {
            assert_eq!(server_value.get(cx), "stable".to_string());
            assert_eq!(
                mutation.get(cx),
                MutationState::Failed {
                    error: "failed",
                    last_success: None,
                }
            );
        });
    }

    #[gpui::test]
    fn optimistic_mutation_cancel_rolls_back_external_state(cx: &mut TestAppContext) {
        let (mutation, server_value) = cx.update(|cx| {
            init(cx);
            (
                use_mutation::<String, &'static str>(cx),
                Signal::new(cx, "stable".to_string()),
            )
        });

        cx.update({
            let mutation_for_run = mutation.clone();
            let server_value = server_value.clone();
            move |cx| {
                mutation_for_run.run_optimistic(
                    cx,
                    move |cx| {
                        let previous = server_value.get(cx);
                        server_value.set(cx, "optimistic".to_string());
                        move |cx| {
                            server_value.set(cx, previous);
                        }
                    },
                    |cx| async move {
                        cx.background_executor()
                            .timer(Duration::from_millis(20))
                            .await;
                        Ok("server".to_string())
                    },
                );
                mutation_for_run.cancel(cx);
            }
        });

        cx.read(|cx| {
            assert_eq!(server_value.get(cx), "stable".to_string());
            assert_eq!(mutation.get(cx), MutationState::Idle);
        });
    }

    #[gpui::test]
    fn optimistic_mutation_success_can_reconcile_with_followup(cx: &mut TestAppContext) {
        let (mutation, server_value) = cx.update(|cx| {
            init(cx);
            (
                use_mutation::<String, &'static str>(cx),
                Signal::new(cx, "stable".to_string()),
            )
        });

        cx.update({
            let mutation_for_run = mutation.clone();
            let server_value_for_optimistic = server_value.clone();
            let server_value_for_followup = server_value.clone();
            move |cx| {
                mutation_for_run.run_optimistic_with_followup(
                    cx,
                    move |cx| {
                        let previous = server_value_for_optimistic.get(cx);
                        server_value_for_optimistic.set(cx, "optimistic".to_string());
                        move |cx| {
                            server_value_for_optimistic.set(cx, previous);
                        }
                    },
                    |cx| async move {
                        cx.background_executor()
                            .timer(Duration::from_millis(20))
                            .await;
                        Ok("canonical".to_string())
                    },
                    move |cx, mutation| {
                        if let Some(saved) = mutation.last_success_value(cx) {
                            server_value_for_followup.set(cx, saved);
                        }
                    },
                );
            }
        });

        cx.read(|cx| {
            assert_eq!(server_value.get(cx), "optimistic".to_string());
        });

        cx.executor().advance_clock(Duration::from_millis(20));
        cx.run_until_parked();

        cx.read(|cx| {
            assert_eq!(server_value.get(cx), "canonical".to_string());
            assert_eq!(
                mutation.get(cx),
                MutationState::Succeeded("canonical".to_string())
            );
        });
    }

    #[gpui::test]
    fn newer_run_rolls_back_previous_optimistic_state_before_applying_next(
        cx: &mut TestAppContext,
    ) {
        let (mutation, server_value) = cx.update(|cx| {
            init(cx);
            (
                use_mutation::<String, &'static str>(cx),
                Signal::new(cx, "stable".to_string()),
            )
        });
        let seen_before_second_apply = Rc::new(RefCell::new(Vec::new()));

        cx.update({
            let mutation_for_first_run = mutation.clone();
            let server_value = server_value.clone();
            move |cx| {
                mutation_for_first_run.run_optimistic(
                    cx,
                    move |cx| {
                        let previous = server_value.get(cx);
                        server_value.set(cx, "first".to_string());
                        move |cx| {
                            server_value.set(cx, previous);
                        }
                    },
                    |cx| async move {
                        cx.background_executor()
                            .timer(Duration::from_millis(20))
                            .await;
                        Ok("first-result".to_string())
                    },
                );
            }
        });

        cx.update({
            let mutation_for_second_run = mutation.clone();
            let server_value = server_value.clone();
            let seen_before_second_apply = seen_before_second_apply.clone();
            move |cx| {
                mutation_for_second_run.run_optimistic(
                    cx,
                    move |cx| {
                        seen_before_second_apply
                            .borrow_mut()
                            .push(server_value.get(cx));
                        let previous = server_value.get(cx);
                        server_value.set(cx, "second".to_string());
                        move |cx| {
                            server_value.set(cx, previous);
                        }
                    },
                    |_| async move { Ok("second-result".to_string()) },
                );
            }
        });

        cx.run_until_parked();

        cx.read(|cx| {
            assert_eq!(&*seen_before_second_apply.borrow(), &["stable".to_string()]);
            assert_eq!(server_value.get(cx), "second".to_string());
            assert_eq!(
                mutation.get(cx),
                MutationState::Succeeded("second-result".to_string())
            );
        });

        cx.executor().advance_clock(Duration::from_millis(20));
        cx.run_until_parked();

        cx.read(|cx| {
            assert_eq!(server_value.get(cx), "second".to_string());
            assert_eq!(
                mutation.get(cx),
                MutationState::Succeeded("second-result".to_string())
            );
        });
    }
}
