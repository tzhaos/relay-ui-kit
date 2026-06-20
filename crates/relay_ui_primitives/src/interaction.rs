//! View-free callback types for Relay components.
//!
//! Instead of GPUI's `Entity<X>`-coupled listeners, every component accepts plain
//! `Box<dyn Fn>` closures.  This keeps the component library decoupled from any
//! concrete app view and lets the gallery and the real workbench wire the same
//! component to different handlers.
//!
//! # Box vs Rc
//!
//! Use `Box` for single-consumption handlers (`RenderOnce` components that fire the
//! handler once per render).  Use `Rc` when a handler must be cloned into multiple
//! sub-elements (e.g. a split-pane handle that renders two drag zones).

use std::rc::Rc;

use gpui::{App, ClickEvent, Hsla, KeyDownEvent, Window};

// ---------------------------------------------------------------------------
// Mouse click handlers
// ---------------------------------------------------------------------------

/// A simple click callback (`on_click`).  Receives the raw [`ClickEvent`].
pub type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Shared variant of [`ClickHandler`] for handlers that are cloned across
/// multiple elements.
pub type SharedClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

// ---------------------------------------------------------------------------
// Overlay / dialog lifecycle handlers
// ---------------------------------------------------------------------------

/// Fires when an overlay or dialog is dismissed (click-outside, Escape key).
pub type DismissHandler = Box<dyn Fn(&mut Window, &mut App) + 'static>;
pub type SharedDismissHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

/// Fires on form submit (Enter in a focused input, submit button).
pub type SubmitHandler = Box<dyn Fn(&mut Window, &mut App) + 'static>;
pub type SharedSubmitHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

/// Fires when an interaction is explicitly cancelled (Escape, cancel button).
pub type CancelHandler = Box<dyn Fn(&mut Window, &mut App) + 'static>;
pub type SharedCancelHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

// ---------------------------------------------------------------------------
// Selection handlers
// ---------------------------------------------------------------------------

/// Fires when a value is selected from a list, menu, or picker.  Receives a
/// compile-time string key identifying the selected item.
pub type SelectHandler = Box<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;
pub type SharedSelectHandler = Rc<dyn Fn(&'static str, &mut Window, &mut App) + 'static>;

// ---------------------------------------------------------------------------
// Generic action / change handlers
// ---------------------------------------------------------------------------

/// Generic action dispatch.  Use when a component emits a single action value
/// (e.g. a button press with associated data).
pub type ActionHandler<T> = Box<dyn Fn(T, &mut Window, &mut App) + 'static>;
pub type SharedActionHandler<T> = Rc<dyn Fn(T, &mut Window, &mut App) + 'static>;

/// Value-change notification.  Fires whenever the host-owned state is mutated
/// (text changed, slider moved, toggle flipped).
pub type ChangeHandler<T> = Box<dyn Fn(T, &mut Window, &mut App) + 'static>;
pub type SharedChangeHandler<T> = Rc<dyn Fn(T, &mut Window, &mut App) + 'static>;

// ---------------------------------------------------------------------------
// Keyboard handlers
// ---------------------------------------------------------------------------

/// Fires on every `keydown` event when the element has focus.
pub type KeyHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;

/// Like [`KeyHandler`], but returns `true` when the event was consumed
/// (stops propagation).  Returning `false` lets the event bubble.
pub type KeyCaptureHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) -> bool + 'static>;

// ---------------------------------------------------------------------------
// Color picker handler
// ---------------------------------------------------------------------------

/// Fires when a color is selected from a color picker.  Receives the named
/// key and the chosen [`Hsla`] value.
pub type ColorSelectHandler = Box<dyn Fn(&'static str, Hsla, &mut Window, &mut App) + 'static>;
pub type SharedColorSelectHandler = Rc<dyn Fn(&'static str, Hsla, &mut Window, &mut App) + 'static>;

// ---------------------------------------------------------------------------
// Callback builder macros
// ---------------------------------------------------------------------------

/// Generate a `Box`-based callback builder method.
///
/// Usage inside a component `impl` block:
/// ```ignore
/// callback_builder!(on_click, on_click, &ClickEvent);
/// ```
///
/// Expands to:
/// ```ignore
/// pub fn on_click(
///     mut self,
///     handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
/// ) -> Self {
///     self.on_click = Some(Box::new(handler));
///     self
/// }
/// ```
#[macro_export]
macro_rules! callback_builder {
    ($method:ident, $field:ident, $($arg:ty),+ $(,)?) => {
        #[doc = concat!(
            "Attach a handler that receives `(",
            $(stringify!($arg), ", ",)+
            "&mut Window, &mut App)`."
        )]
        pub fn $method(
            mut self,
            handler: impl Fn($($arg,)+ &mut ::gpui::Window, &mut ::gpui::App) + 'static,
        ) -> Self {
            self.$field = Some(Box::new(handler));
            self
        }
    };
}

/// Generate an `Rc`-based shared callback builder method.
///
/// Usage:
/// ```ignore
/// shared_callback_builder!(on_change, on_change, f32);
/// ```
#[macro_export]
macro_rules! shared_callback_builder {
    ($method:ident, $field:ident, $($arg:ty),+ $(,)?) => {
        #[doc = concat!(
            "Attach a shared handler that receives `(",
            $(stringify!($arg), ", ",)+
            "&mut Window, &mut App)`."
        )]
        pub fn $method(
            mut self,
            handler: impl Fn($($arg,)+ &mut ::gpui::Window, &mut ::gpui::App) + 'static,
        ) -> Self {
            self.$field = Some(::std::rc::Rc::new(handler));
            self
        }
    };
}
