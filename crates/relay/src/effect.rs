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
