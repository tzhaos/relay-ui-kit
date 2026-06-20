//! GPUI-native reactive state runtime.
//!
//!
//! # Example
//!
//! ```rust,no_run
//! use gpui::{Context, IntoElement, Render, Window, div, prelude::*};
//! use relay::{Signal, init, track};
//!
//! struct Counter {
//!     count: Signal<i32>,
//! }
//!
//! impl Counter {
//!     fn new(cx: &mut Context<Self>) -> Self {
//!         init(cx);
//!         Self {
//!             count: Signal::new(cx, 0),
//!         }
//!     }
//! }
//!
//! impl Render for Counter {
//!     fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//!         track(cx, |cx| div().child(self.count.get(cx).to_string()))
//!     }
//! }
//! ```

mod binding;
mod effect;
mod memo;
mod resource;
mod runtime;
mod signal;
mod window_ext;

pub use binding::Binding;
pub use effect::{Effect, effect, effect_in};
pub use memo::Memo;
pub use resource::{Resource, ResourceState};
pub use runtime::{EffectId, ReactiveRuntime, SignalId, batch, init, is_installed, track};
pub use signal::{ReadSignal, Signal, WriteSignal};
pub use window_ext::WindowSignalExt;
