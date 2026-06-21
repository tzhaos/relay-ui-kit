use std::{cell::RefCell, rc::Rc};

use gpui::{App, Context};

use crate::{EffectId, ReactiveRuntime, init};

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
        if let Some(runtime) = cx.try_global::<ReactiveRuntime>() {
            runtime.remove_effect(self.id);
        }
    }
}

/// Create an app-scoped reactive effect.
///
/// The effect runs immediately to discover dependencies. When any dependency
/// changes, relay runs the effect again.
pub fn effect(cx: &mut App, f: impl FnMut(&mut App) + 'static) -> Effect {
    init(cx);
    let callback = Rc::new(RefCell::new(f));
    let id = cx.global::<ReactiveRuntime>().insert_effect(callback);
    ReactiveRuntime::run_effect(cx, id);
    Effect { id }
}

/// Create an effect scoped to the current GPUI entity.
pub fn effect_in<T: 'static>(cx: &mut Context<T>, f: impl FnMut(&mut App) + 'static) -> Effect {
    let effect = effect(cx, f);
    let effect_id = effect.id;
    let subscription = cx.on_release(move |_, cx| {
        if let Some(runtime) = cx.try_global::<ReactiveRuntime>() {
            runtime.remove_effect(effect_id);
        }
    });
    cx.global::<ReactiveRuntime>()
        .insert_effect_release_subscription(effect_id, subscription);
    effect
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::TestApp;

    use crate::{Signal, effect, effect_in, init};

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
            seen: Rc<Cell<i32>>,
            _effect: crate::Effect,
        }

        impl EffectHost {
            fn new(cx: &mut Context<Self>, seen: Rc<Cell<i32>>, signal: Signal<i32>) -> Self {
                let seen_for_effect = seen.clone();
                let _effect = effect_in(cx, move |cx| {
                    seen_for_effect.set(signal.get(cx));
                });
                Self { seen, _effect }
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
        app.update(|cx| init(cx));

        let e1 = app.update(|cx| effect(cx, |_| {}));
        let e2 = app.update(|cx| effect(cx, |_| {}));

        assert_ne!(e1.id(), e2.id(), "each effect should have a unique id");
    }
}
