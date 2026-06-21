//! Reactive context — provide/inject for signal-backed shared state.
//!
//! A context is a named, signal-backed slot stored as a GPUI global. Producers
//! call [`provide_context`] to install a value; consumers call [`use_context`]
//! to read it with dependency tracking. When the provided value changes, all
//! views that read it via `use_context` are notified.
//!
//! This replaces prop-drilling for cross-layer reactive state such as theme,
//! locale, or the currently active workspace entity.

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gpui::{App, Global};

use crate::{ReactiveRuntime, init};

/// Opaque handle to an installed context slot.
///
/// Each call to [`provide_context`] returns a `ContextHandle` that can be used
/// to update the value later via [`ContextHandle::set`].
pub struct ContextHandle<T: 'static> {
    slot: Rc<ContextSlot<T>>,
    signal_id: crate::SignalId,
}

impl<T: 'static> ContextHandle<T> {
    /// Replace the context value and notify all consumers.
    pub fn set(&self, cx: &mut App, value: T)
    where
        T: PartialEq,
    {
        let changed = {
            let mut value_ref = self.slot.value.borrow_mut();
            if value_ref.as_ref() == Some(&value) {
                false
            } else {
                *value_ref = Some(value);
                true
            }
        };
        if changed {
            let notifications = cx.global::<ReactiveRuntime>().notify_signal(self.signal_id);
            ReactiveRuntime::flush_notifications(cx, notifications);
        }
    }
}

impl<T: 'static> Clone for ContextHandle<T> {
    fn clone(&self) -> Self {
        Self {
            slot: self.slot.clone(),
            signal_id: self.signal_id,
        }
    }
}

/// Type-erased storage for a context slot, bundling the value and its signal id.
struct ContextSlot<T> {
    value: RefCell<Option<T>>,
    signal_id: crate::SignalId,
}

/// Internal registry mapping `TypeId` to type-erased context slots.
#[derive(Default)]
struct ContextRegistry {
    slots: RefCell<HashMap<TypeId, Rc<dyn Any>>>,
}

impl Global for ContextRegistry {}

impl ContextRegistry {
    fn get_or_create_slot<T: 'static>(&self, cx: &App) -> Rc<ContextSlot<T>> {
        let mut slots = self.slots.borrow_mut();
        slots
            .entry(TypeId::of::<T>())
            .or_insert_with(|| {
                let signal_id = cx.global::<ReactiveRuntime>().allocate_signal();
                Rc::new(ContextSlot::<T> {
                    value: RefCell::new(None),
                    signal_id,
                })
            })
            .clone()
            .downcast::<ContextSlot<T>>()
            .expect("type mismatch in context slot")
    }
}

/// Provide a reactive context value for type `T`.
///
/// Subsequent calls overwrite the value and notify consumers. Returns a handle
/// that can update the value later without re-providing.
pub fn provide_context<T: 'static>(cx: &mut App, value: T) -> ContextHandle<T> {
    init(cx);
    if !cx.has_global::<ContextRegistry>() {
        cx.set_global(ContextRegistry::default());
    }
    let slot = cx.global::<ContextRegistry>().get_or_create_slot::<T>(cx);
    {
        let mut value_ref = slot.value.borrow_mut();
        *value_ref = Some(value);
    }
    // Notify immediately so consumers created before this call refresh.
    let notifications = cx.global::<ReactiveRuntime>().notify_signal(slot.signal_id);
    ReactiveRuntime::flush_notifications(cx, notifications);

    let signal_id = slot.signal_id;
    ContextHandle { slot, signal_id }
}

/// Read a context value with dependency tracking.
///
/// Returns `Some` if [`provide_context`] was called for `T`; otherwise `None`.
/// The caller is subscribed to the context's signal and will be notified when
/// the value changes.
pub fn use_context<T: Clone + 'static>(cx: &App) -> Option<T> {
    // Don't call init() here — use_context is read-only and the runtime
    // should already be installed by provide_context or the app's startup.
    let registry = cx.try_global::<ContextRegistry>()?;
    let slot = registry.get_or_create_slot::<T>(cx);
    // Track the context's signal so the caller is notified on change.
    cx.global::<ReactiveRuntime>().track_signal(slot.signal_id);
    slot.value.borrow().clone()
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use gpui::{Context, IntoElement, ParentElement, Render, TestApp, Window, div};

    use crate::{init, track};

    use super::*;

    struct Consumer {
        theme_handle: ContextHandle<String>,
    }

    impl Consumer {
        fn new(cx: &mut Context<Self>) -> Self {
            init(cx);
            let theme_handle = provide_context(cx, "light".to_string());
            Self { theme_handle }
        }
    }

    impl Render for Consumer {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            track(cx, |cx| {
                let theme = use_context::<String>(cx).unwrap_or_default();
                div().child(theme)
            })
        }
    }

    #[test]
    fn context_value_can_be_read() {
        let mut app = TestApp::new();
        let theme = app.update(|cx| {
            init(cx);
            provide_context(cx, "dark".to_string())
        });

        app.read(|cx| {
            let value = use_context::<String>(cx);
            assert_eq!(value, Some("dark".to_string()));
        });

        let _ = theme;
    }

    #[test]
    fn context_set_notifies_consumer() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| Consumer::new(cx));
        let root = window.root();
        window.draw();

        let notifications = Rc::new(Cell::new(0));
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
            view.theme_handle.set(cx, "dark".to_string());
        });

        assert_eq!(notifications.get(), 1);
    }

    #[test]
    fn context_set_same_value_does_not_notify() {
        let mut app = TestApp::new();
        let mut window = app.open_window(|_, cx| Consumer::new(cx));
        let root = window.root();
        window.draw();

        let notifications = Rc::new(Cell::new(0));
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
            view.theme_handle.set(cx, "light".to_string());
        });

        assert_eq!(notifications.get(), 0);
    }

    #[test]
    fn use_context_returns_none_when_not_provided() {
        let mut app = TestApp::new();
        app.update(|cx| init(cx));

        app.read(|cx| {
            let value = use_context::<i32>(cx);
            assert_eq!(value, None);
        });
    }
}
