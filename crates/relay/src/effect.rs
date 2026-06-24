use std::{cell::RefCell, rc::Rc};

use gpui::{App, Context};

use crate::{EffectId, ReactiveRuntime, init};

pub(crate) type EffectCleanup = Box<dyn FnOnce(&mut App)>;

/// Cleanup registration for one effect run.
///
/// Register cleanups from [`effect_with_cleanup`] or
/// [`effect_in_with_cleanup`] when a side effect owns a temporary handle, such
/// as a subscription, listener, or source-dependent task. Cleanups run before
/// the effect re-runs and when the effect is disposed.
pub struct CleanupScope {
    cleanups: Vec<EffectCleanup>,
}

impl CleanupScope {
    pub(crate) fn new() -> Self {
        Self {
            cleanups: Vec::new(),
        }
    }

    /// Register a cleanup for the current effect run.
    pub fn on_cleanup(&mut self, cleanup: impl FnOnce(&mut App) + 'static) {
        self.cleanups.push(Box::new(cleanup));
    }

    pub(crate) fn into_cleanups(self) -> Vec<EffectCleanup> {
        self.cleanups
    }
}

/// Handle to a registered reactive effect.
///
/// Effects created with [`effect_in`] are also removed when their owner GPUI
/// entity is released. Use [`Effect::dispose`] for manual cleanup.
pub struct Effect {
    id: EffectId,
}

impl Effect {
    /// Return this effect's runtime identity.
    pub fn id(&self) -> EffectId {
        self.id
    }

    /// Create an effect handle from a known id (crate-internal use only).
    pub(crate) fn from_id(id: EffectId) -> Self {
        Self { id }
    }

    /// Remove the effect and unsubscribe it from all signals.
    pub fn dispose(self, cx: &mut App) {
        ReactiveRuntime::remove_effect(cx, self.id);
    }
}

/// Create an app-scoped reactive effect.
///
/// The effect runs immediately to discover dependencies. When any dependency
/// changes, relay runs the effect again.
pub fn effect(cx: &mut App, mut f: impl FnMut(&mut App) + 'static) -> Effect {
    effect_with_cleanup(cx, move |cx, _cleanup| f(cx))
}

/// Create an app-scoped reactive effect with per-run cleanup.
///
/// Cleanups registered during a run execute before the next run and when the
/// effect is disposed. Cleanup signal reads are untracked; cleanup writes still
/// notify normally.
pub fn effect_with_cleanup(
    cx: &mut App,
    f: impl FnMut(&mut App, &mut CleanupScope) + 'static,
) -> Effect {
    init(cx);
    let callback = Rc::new(RefCell::new(f));
    let id = cx.global::<ReactiveRuntime>().insert_effect(callback);
    ReactiveRuntime::run_effect(cx, id);
    Effect { id }
}

/// Create an effect scoped to the current GPUI entity.
pub fn effect_in<T: 'static>(cx: &mut Context<T>, mut f: impl FnMut(&mut App) + 'static) -> Effect {
    effect_in_with_cleanup(cx, move |cx, _cleanup| f(cx))
}

/// Create an entity-scoped reactive effect with per-run cleanup.
pub fn effect_in_with_cleanup<T: 'static>(
    cx: &mut Context<T>,
    f: impl FnMut(&mut App, &mut CleanupScope) + 'static,
) -> Effect {
    let effect = effect_with_cleanup(cx, f);
    let effect_id = effect.id;
    let subscription = cx.on_release(move |_, cx| {
        ReactiveRuntime::remove_effect(cx, effect_id);
    });
    cx.global::<ReactiveRuntime>()
        .insert_effect_release_subscription(effect_id, subscription);
    effect
}

#[cfg(test)]
mod tests {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    use gpui::TestApp;

    use crate::{Signal, effect, effect_in, effect_in_with_cleanup, effect_with_cleanup, init};

    #[test]
    fn effect_dispose_removes_callback() {
        let mut app = TestApp::new();
        let signal = app.update(|cx| {
            init(cx);
            Signal::new(cx, 0)
        });

        let runs = Rc::new(Cell::new(0));
        let effect_handle = app.update({
            let runs = runs.clone();
            let signal = signal.clone();
            move |cx| {
                effect(cx, move |cx| {
                    let _ = signal.get(cx);
                    runs.set(runs.get() + 1);
                })
            }
        });

        assert_eq!(runs.get(), 1, "effect runs once on creation");

        app.update(|cx| effect_handle.dispose(cx));

        app.update(|cx| signal.set(cx, 5));
        assert_eq!(runs.get(), 1, "disposed effect should not rerun");
    }

    #[test]
    fn effect_in_is_scoped_to_entity() {
        // effect_in creates an effect that is disposed when the entity is
        // released. We verify the effect runs initially and responds to
        // signal changes.
        use gpui::{Context, IntoElement, Render, Window, div};

        struct EffectHost {
            _effect: crate::Effect,
        }

        impl EffectHost {
            fn new(cx: &mut Context<Self>, seen: Rc<Cell<i32>>, signal: Signal<i32>) -> Self {
                let seen_for_effect = seen.clone();
                let _effect = effect_in(cx, move |cx| {
                    seen_for_effect.set(signal.get(cx));
                });
                Self { _effect }
            }
        }

        impl Render for EffectHost {
            fn render(&mut self, _w: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
                div()
            }
        }

        let mut app = TestApp::new();
        let signal = app.update(|cx| {
            init(cx);
            Signal::new(cx, 42)
        });

        let seen = Rc::new(Cell::new(0));
        let signal_for_entity = signal.clone();

        let mut window = app.open_window(|_, cx| {
            let seen = seen.clone();
            let signal = signal_for_entity.clone();
            EffectHost::new(cx, seen, signal)
        });
        let root = window.root();
        window.draw();

        assert_eq!(seen.get(), 42, "effect_in runs on creation");

        app.update(|cx| signal.set(cx, 100));
        assert_eq!(seen.get(), 100);

        let _ = root;
    }

    #[test]
    fn effect_id_is_unique() {
        let mut app = TestApp::new();
        app.update(init);

        let e1 = app.update(|cx| effect(cx, |_| {}));
        let e2 = app.update(|cx| effect(cx, |_| {}));

        assert_ne!(e1.id(), e2.id(), "each effect should have a unique id");
    }

    #[test]
    fn effect_cleanup_runs_before_rerun() {
        let mut app = TestApp::new();
        let signal = app.update(|cx| {
            init(cx);
            Signal::new(cx, 0)
        });
        let events = Rc::new(RefCell::new(Vec::new()));

        let _effect = app.update({
            let signal = signal.clone();
            let events = events.clone();
            move |cx| {
                effect_with_cleanup(cx, move |cx, cleanup| {
                    let value = signal.get(cx);
                    events.borrow_mut().push(format!("run {value}"));
                    let events = events.clone();
                    cleanup.on_cleanup(move |_cx| {
                        events.borrow_mut().push(format!("cleanup {value}"));
                    });
                })
            }
        });

        app.update(|cx| signal.set(cx, 1));

        assert_eq!(
            &*events.borrow(),
            &["run 0", "cleanup 0", "run 1"],
            "previous cleanup should run before the next effect body"
        );
    }

    #[test]
    fn effect_cleanup_runs_on_dispose() {
        let mut app = TestApp::new();
        let cleanup_count = Rc::new(Cell::new(0));

        let effect_handle = app.update({
            let cleanup_count = cleanup_count.clone();
            move |cx| {
                effect_with_cleanup(cx, move |_cx, cleanup| {
                    let cleanup_count = cleanup_count.clone();
                    cleanup.on_cleanup(move |_cx| {
                        cleanup_count.set(cleanup_count.get() + 1);
                    });
                })
            }
        });

        assert_eq!(cleanup_count.get(), 0);

        app.update(|cx| effect_handle.dispose(cx));

        assert_eq!(cleanup_count.get(), 1);
    }

    #[test]
    fn cleanup_reads_are_untracked() {
        let mut app = TestApp::new();
        let (source, cleanup_only) = app.update(|cx| {
            init(cx);
            (Signal::new(cx, 0), Signal::new(cx, 0))
        });
        let runs = Rc::new(Cell::new(0));

        let _effect = app.update({
            let source = source.clone();
            let cleanup_only = cleanup_only.clone();
            let runs = runs.clone();
            move |cx| {
                effect_with_cleanup(cx, move |cx, cleanup| {
                    let _ = source.get(cx);
                    runs.set(runs.get() + 1);
                    let cleanup_only = cleanup_only.clone();
                    cleanup.on_cleanup(move |cx| {
                        let _ = cleanup_only.get(cx);
                    });
                })
            }
        });

        app.update(|cx| source.set(cx, 1));
        assert_eq!(runs.get(), 2);

        app.update(|cx| cleanup_only.set(cx, 1));
        assert_eq!(runs.get(), 2);
    }

    #[test]
    fn entity_scoped_effect_cleanup_runs_on_release() {
        use gpui::{AppContext, Context};

        struct CleanupHost {
            _effect: crate::Effect,
        }

        impl CleanupHost {
            fn new(cx: &mut Context<Self>, cleanup_count: Rc<Cell<i32>>) -> Self {
                let effect = effect_in_with_cleanup(cx, move |_cx, cleanup| {
                    let cleanup_count = cleanup_count.clone();
                    cleanup.on_cleanup(move |_cx| {
                        cleanup_count.set(cleanup_count.get() + 1);
                    });
                });
                Self { _effect: effect }
            }
        }

        let mut app = TestApp::new();
        let cleanup_count = Rc::new(Cell::new(0));
        let weak = app.update({
            let cleanup_count = cleanup_count.clone();
            move |cx| {
                let entity = cx.new(|cx| CleanupHost::new(cx, cleanup_count));
                let weak = entity.downgrade();
                drop(entity);
                weak
            }
        });

        weak.assert_released();
        assert_eq!(cleanup_count.get(), 1);
    }
}
