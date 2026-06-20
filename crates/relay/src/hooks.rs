//! GPUI-friendly helpers for creating relay state.

use std::borrow::BorrowMut;

use gpui::{App, Context};

use crate::{Binding, Effect, Memo, Resource, Signal, effect_in, init, track};

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
}

/// Reactive helpers that need the current GPUI entity.
pub trait ReactiveContextExt<T: 'static> {
    /// Render while tracking signal reads for the current entity.
    fn tracked<R>(&mut self, render: impl FnOnce(&mut Context<T>) -> R) -> R;

    /// Create an effect that is disposed when the current entity is released.
    fn effect_scoped(&mut self, f: impl FnMut(&mut App) + 'static) -> Effect;
}

impl<T: 'static> ReactiveContextExt<T> for Context<'_, T> {
    fn tracked<R>(&mut self, render: impl FnOnce(&mut Context<T>) -> R) -> R {
        track(self, render)
    }

    fn effect_scoped(&mut self, f: impl FnMut(&mut App) + 'static) -> Effect {
        effect_in(self, f)
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
}
