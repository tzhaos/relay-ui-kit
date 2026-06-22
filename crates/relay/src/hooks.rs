//! GPUI-friendly helpers for creating relay state.

use std::{borrow::BorrowMut, cell::Cell, hash::Hash};

use gpui::{App, Context};

use crate::{
    Binding, CleanupScope, Effect, Memo, Resource, Selector, Signal, effect_in,
    effect_in_with_cleanup, init, track, untrack,
};

/// Convenience constructors for relay primitives on GPUI app contexts.
pub trait ReactiveAppExt {
    /// Create a read/write signal.
    fn signal<T>(&mut self, value: T) -> Signal<T>;

    /// Create a two-way binding backed by a signal.
    fn binding<T>(&mut self, value: T) -> Binding<T>;

    /// Create a derived value that updates when its dependencies change.
    fn memo<T>(&mut self, compute: impl Fn(&App) -> T + 'static) -> Memo<T>
    where
        T: PartialEq + 'static;

    /// Create a pending async resource.
    fn pending_resource<T, E>(&mut self) -> Resource<T, E>;

    /// Create a ready async resource.
    fn ready_resource<T, E>(&mut self, value: T) -> Resource<T, E>;

    /// Create an errored async resource.
    fn error_resource<T, E>(&mut self, error: E) -> Resource<T, E>;

    /// Create keyed selection state.
    fn selector<K>(&mut self, selected: Option<K>) -> Selector<K>
    where
        K: Clone + Eq + Hash + PartialEq + 'static;

    /// Run a closure without recording signal reads as dependencies.
    ///
    /// Convenience wrapper around [`crate::untrack`].
    fn untrack<R>(&mut self, f: impl FnOnce(&mut App) -> R) -> R;

    /// Batch signal writes and flush affected observers once at the end.
    ///
    /// This is a convenience wrapper around [`crate::batch`].
    fn batch<R>(&mut self, f: impl FnOnce(&mut App) -> R) -> R;

    /// Create a derived value that updates when its dependencies change.
    ///
    /// This is a semantic alias for [`ReactiveAppExt::memo`], intended for call
    /// sites where the value expresses a *derivation* (e.g. filtered list,
    /// selected item detail, form validity) rather than a generic cached
    /// computation. Use `derived` in application code for clarity.
    fn derived<T>(&mut self, compute: impl Fn(&App) -> T + 'static) -> Memo<T>
    where
        T: PartialEq + 'static,
    {
        self.memo(compute)
    }
}

impl<C> ReactiveAppExt for C
where
    C: BorrowMut<App>,
{
    fn signal<T>(&mut self, value: T) -> Signal<T> {
        Signal::new(self.borrow_mut(), value)
    }

    fn binding<T>(&mut self, value: T) -> Binding<T> {
        Binding::from(self.signal(value))
    }

    fn memo<T>(&mut self, compute: impl Fn(&App) -> T + 'static) -> Memo<T>
    where
        T: PartialEq + 'static,
    {
        Memo::new(self.borrow_mut(), compute)
    }

    fn pending_resource<T, E>(&mut self) -> Resource<T, E> {
        Resource::pending(self.borrow_mut())
    }

    fn ready_resource<T, E>(&mut self, value: T) -> Resource<T, E> {
        Resource::ready(self.borrow_mut(), value)
    }

    fn error_resource<T, E>(&mut self, error: E) -> Resource<T, E> {
        Resource::error(self.borrow_mut(), error)
    }

    fn selector<K>(&mut self, selected: Option<K>) -> Selector<K>
    where
        K: Clone + Eq + Hash + PartialEq + 'static,
    {
        Selector::new(self.borrow_mut(), selected)
    }

    fn untrack<R>(&mut self, f: impl FnOnce(&mut App) -> R) -> R {
        untrack(self.borrow_mut(), f)
    }

    fn batch<R>(&mut self, f: impl FnOnce(&mut App) -> R) -> R {
        crate::batch(self.borrow_mut(), f)
    }
}

/// Reactive helpers that need the current GPUI entity.
pub trait ReactiveContextExt<T: 'static> {
    /// Render while tracking signal reads for the current entity.
    fn tracked<R>(&mut self, render: impl FnOnce(&mut Context<T>) -> R) -> R;

    /// Create an effect that is disposed when the current entity is released.
    fn effect_scoped(&mut self, f: impl FnMut(&mut App) + 'static) -> Effect;

    /// Create an effect that is disposed when the current entity is released.
    ///
    /// The closure runs immediately and re-runs whenever any signal it reads
    /// changes. Prefer this over `effect_scoped` when the side effect should
    /// read signals and react to them — it documents intent at the call site.
    fn effect_in(&mut self, f: impl FnMut(&mut App) + 'static) -> Effect;

    /// Create an entity-scoped effect with cleanup before re-run and release.
    fn effect_in_with_cleanup(
        &mut self,
        f: impl FnMut(&mut App, &mut CleanupScope) + 'static,
    ) -> Effect;

    /// Watch signals for changes and run a side effect.
    ///
    /// `sources` reads the dependencies (and is re-evaluated each run so
    /// conditional dependencies work). `react` runs the side effect after the
    /// sources are collected and is wrapped in `untrack`, so reads inside the
    /// side effect do not become dependencies. The effect is scoped to the
    /// current entity and disposed when it is released.
    ///
    /// Unlike [`ReactiveContextExt::effect_in`], this separates dependency
    /// declaration from the side effect, making "when X changes, do Y" explicit.
    fn watch<S, R>(&mut self, sources: S, react: R) -> Effect
    where
        S: Fn(&App) + 'static,
        R: Fn(&mut App) + 'static;

    /// Watch signals for changes without firing the side effect on creation.
    ///
    /// This is useful when the initial visible state is already seeded, but
    /// later source changes should trigger a reload, sync, or other side
    /// effect.
    fn watch_changes<S, R>(&mut self, sources: S, react: R) -> Effect
    where
        S: Fn(&App) + 'static,
        R: Fn(&mut App) + 'static;
}

impl<T: 'static> ReactiveContextExt<T> for Context<'_, T> {
    fn tracked<R>(&mut self, render: impl FnOnce(&mut Context<T>) -> R) -> R {
        track(self, render)
    }

    fn effect_scoped(&mut self, f: impl FnMut(&mut App) + 'static) -> Effect {
        effect_in(self, f)
    }

    fn effect_in(&mut self, f: impl FnMut(&mut App) + 'static) -> Effect {
        effect_in(self, f)
    }

    fn effect_in_with_cleanup(
        &mut self,
        f: impl FnMut(&mut App, &mut CleanupScope) + 'static,
    ) -> Effect {
        effect_in_with_cleanup(self, f)
    }

    fn watch<S, R>(&mut self, sources: S, react: R) -> Effect
    where
        S: Fn(&App) + 'static,
        R: Fn(&mut App) + 'static,
    {
        effect_in(self, move |cx| {
            (sources)(cx);
            untrack(cx, |cx| (react)(cx));
        })
    }

    fn watch_changes<S, R>(&mut self, sources: S, react: R) -> Effect
    where
        S: Fn(&App) + 'static,
        R: Fn(&mut App) + 'static,
    {
        let initialized = Cell::new(false);
        effect_in(self, move |cx| {
            (sources)(cx);
            if initialized.replace(true) {
                untrack(cx, |cx| (react)(cx));
            }
        })
    }
}

/// Install relay into the app.
pub fn install(cx: &mut App) {
    init(cx);
}

#[cfg(test)]
mod tests {
    use gpui::TestApp;

    use crate::{ReactiveAppExt, ResourceState};

    #[test]
    fn app_ext_creates_binding() {
        let mut app = TestApp::new();
        let binding = app.update(|cx| cx.binding(false));

        app.update(|cx| binding.set(cx, true));

        app.read(|cx| {
            assert!(binding.get(cx));
        });
    }

    #[test]
    fn app_ext_creates_ready_resource() {
        let mut app = TestApp::new();
        let resource = app.update(|cx| cx.ready_resource::<_, &'static str>(7));

        app.read(|cx| {
            assert_eq!(resource.get(cx), ResourceState::Ready(7));
        });
    }

    #[test]
    fn app_ext_creates_selector() {
        let mut app = TestApp::new();
        let selector = app.update(|cx| cx.selector(Some(1_u64)));

        app.update(|cx| {
            assert!(selector.is_selected(cx, 1));
            assert!(!selector.is_selected(cx, 2));
        });
    }

    #[test]
    fn app_ext_untrack_does_not_subscribe() {
        use gpui::{Context, IntoElement, Render, Window, div};

        use crate::{Signal, init, track};

        struct UntrackView {
            signal: Signal<i32>,
            other: Signal<i32>,
        }

        impl UntrackView {
            fn new(cx: &mut Context<Self>) -> Self {
                init(cx);
                Self {
                    signal: Signal::new(cx, 0),
                    other: Signal::new(cx, 100),
                }
            }
        }

        impl Render for UntrackView {
            fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                track(cx, |cx| {
                    // tracked read
                    let _a = self.signal.get(cx);
                    // untracked read via ext
                    let _b = cx.untrack(|cx| self.other.get(cx));
                    div()
                })
            }
        }

        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| UntrackView::new(cx));
        let root = window.root();
        window.draw();

        let notifications = std::rc::Rc::new(std::cell::Cell::new(0));
        let _sub = app.update({
            let notifications = notifications.clone();
            let root = root.clone();
            move |cx| {
                cx.observe(&root, move |_, _| {
                    notifications.set(notifications.get() + 1);
                })
            }
        });

        app.update_entity(&root, |view, cx| {
            view.other.set(cx, 200);
        });

        // `other` is untracked, so changing it should not notify.
        assert_eq!(notifications.get(), 0);
    }

    #[test]
    fn app_ext_batch_flushes_effect_once() {
        use std::{cell::Cell, rc::Rc};

        use crate::{effect, init};

        let mut app = TestApp::new();
        let signal = app.update(|cx| {
            init(cx);
            cx.signal(0)
        });

        let runs = Rc::new(Cell::new(0));
        let _effect = app.update({
            let runs = runs.clone();
            let signal = signal.clone();
            move |cx| {
                effect(cx, move |cx| {
                    let _ = signal.get(cx);
                    runs.set(runs.get() + 1);
                })
            }
        });
        assert_eq!(runs.get(), 1);

        app.update(|cx| {
            cx.batch(|cx| {
                signal.set(cx, 1);
                signal.set(cx, 2);
            });
        });

        assert_eq!(runs.get(), 2);
    }

    #[test]
    fn derived_updates_when_source_changes() {
        let mut app = TestApp::new();
        let (source, derived) = app.update(|cx| {
            crate::init(cx);
            let source: crate::Signal<i32> = cx.signal(3);
            let derived = cx.derived({
                let source = source.clone();
                move |cx| source.get(cx) * 2
            });
            (source, derived)
        });

        app.read(|cx| {
            assert_eq!(derived.get(cx), 6);
        });

        app.update(|cx| source.set(cx, 5));

        app.read(|cx| {
            assert_eq!(derived.get(cx), 10);
        });
    }

    #[test]
    fn watch_fires_when_sources_change() {
        use gpui::{Context, IntoElement, ParentElement, Render, Window, div};

        use crate::{ReactiveContextExt, Signal, init, track};

        struct WatchView {
            a: Signal<i32>,
            b: Signal<i32>,
        }

        impl WatchView {
            fn new(cx: &mut Context<Self>, last_sum: std::rc::Rc<std::cell::Cell<i32>>) -> Self {
                init(cx);
                let a = cx.signal(1);
                let b = cx.signal(2);
                let a_for_effect = a.clone();
                let b_for_effect = b.clone();
                let a_for_react = a.clone();
                let b_for_react = b.clone();
                cx.watch(
                    move |cx| {
                        let _ = a_for_effect.get(cx);
                        let _ = b_for_effect.get(cx);
                    },
                    move |cx| {
                        let sum = a_for_react.get(cx) + b_for_react.get(cx);
                        last_sum.set(sum);
                    },
                );
                Self { a, b }
            }
        }

        impl Render for WatchView {
            fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                track(cx, |cx| {
                    div().child((self.a.get(cx) + self.b.get(cx)).to_string())
                })
            }
        }

        let last_sum = std::rc::Rc::new(std::cell::Cell::new(0));
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| WatchView::new(cx, last_sum.clone()));
        window.draw();

        // Initial run of the effect happened during new(); sum should be 3.
        assert_eq!(last_sum.get(), 3);

        app.update_entity(&window.root(), |view, cx| {
            view.a.set(cx, 10);
        });

        // Effect re-ran because `a` (a source) changed.
        assert_eq!(last_sum.get(), 12);
    }

    #[test]
    fn watch_does_not_fire_for_non_source_signals() {
        use gpui::{Context, IntoElement, ParentElement, Render, Window, div};

        use crate::{ReactiveContextExt, Signal, init, track};

        struct WatchView {
            watched: Signal<i32>,
            ignored: Signal<i32>,
        }

        impl WatchView {
            fn new(cx: &mut Context<Self>, fires: std::rc::Rc<std::cell::Cell<i32>>) -> Self {
                init(cx);
                let watched = cx.signal(0);
                let ignored = cx.signal(0);
                let watched_for_effect = watched.clone();
                cx.watch(
                    move |cx| {
                        let _ = watched_for_effect.get(cx);
                    },
                    move |_cx| {
                        fires.set(fires.get() + 1);
                    },
                );
                Self { watched, ignored }
            }
        }

        impl Render for WatchView {
            fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                track(cx, |cx| div().child(self.watched.get(cx).to_string()))
            }
        }

        let fires = std::rc::Rc::new(std::cell::Cell::new(0));
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| WatchView::new(cx, fires.clone()));
        window.draw();

        // Initial run during new() → fires == 1.
        assert_eq!(fires.get(), 1);

        app.update_entity(&window.root(), |view, cx| {
            view.ignored.set(cx, 99);
        });

        // `ignored` is not a source, so the effect should not fire.
        assert_eq!(fires.get(), 1);

        app.update_entity(&window.root(), |view, cx| {
            view.watched.set(cx, 7);
        });

        // `watched` is a source, so the effect fires again.
        assert_eq!(fires.get(), 2);
    }

    #[test]
    fn watch_does_not_track_react_reads() {
        use gpui::{Context, IntoElement, ParentElement, Render, Window, div};

        use crate::{ReactiveContextExt, Signal, init, track};

        struct WatchView {
            watched: Signal<i32>,
            react_only: Signal<i32>,
        }

        impl WatchView {
            fn new(cx: &mut Context<Self>, fires: std::rc::Rc<std::cell::Cell<i32>>) -> Self {
                init(cx);
                let watched = cx.signal(0);
                let react_only = cx.signal(0);
                let watched_for_sources = watched.clone();
                let react_only_for_react = react_only.clone();
                cx.watch(
                    move |cx| {
                        let _ = watched_for_sources.get(cx);
                    },
                    move |cx| {
                        let _ = react_only_for_react.get(cx);
                        fires.set(fires.get() + 1);
                    },
                );
                Self {
                    watched,
                    react_only,
                }
            }
        }

        impl Render for WatchView {
            fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                track(cx, |cx| div().child(self.watched.get(cx).to_string()))
            }
        }

        let fires = std::rc::Rc::new(std::cell::Cell::new(0));
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| WatchView::new(cx, fires.clone()));
        window.draw();

        assert_eq!(fires.get(), 1);

        app.update_entity(&window.root(), |view, cx| {
            view.react_only.set(cx, 1);
        });

        assert_eq!(fires.get(), 1);
    }

    #[test]
    fn watch_changes_skips_initial_react() {
        use gpui::{Context, IntoElement, ParentElement, Render, Window, div};

        use crate::{ReactiveContextExt, Signal, init, track};

        struct WatchView {
            watched: Signal<i32>,
        }

        impl WatchView {
            fn new(cx: &mut Context<Self>, fires: std::rc::Rc<std::cell::Cell<i32>>) -> Self {
                init(cx);
                let watched = cx.signal(0);
                let watched_for_sources = watched.clone();
                cx.watch_changes(
                    move |cx| {
                        let _ = watched_for_sources.get(cx);
                    },
                    move |_cx| {
                        fires.set(fires.get() + 1);
                    },
                );
                Self { watched }
            }
        }

        impl Render for WatchView {
            fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                track(cx, |cx| div().child(self.watched.get(cx).to_string()))
            }
        }

        let fires = std::rc::Rc::new(std::cell::Cell::new(0));
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| WatchView::new(cx, fires.clone()));
        window.draw();

        assert_eq!(fires.get(), 0);

        app.update_entity(&window.root(), |view, cx| {
            view.watched.set(cx, 1);
        });

        assert_eq!(fires.get(), 1);
    }
}
