//! GPUI-native reactive state runtime.
//!
//!
//! # Example
//!
//! ```rust,no_run
//! use gpui::{Context, IntoElement, Render, Window, div, prelude::*};
//! use relay::{ReactiveAppExt, ReactiveContextExt, Signal, init};
//!
//! struct Counter {
//!     count: Signal<i32>,
//! }
//!
//! impl Counter {
//!     fn new(cx: &mut Context<Self>) -> Self {
//!         init(cx);
//!         Self {
//!             count: cx.signal(0),
//!         }
//!     }
//! }
//!
//! impl Render for Counter {
//!     fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//!         cx.tracked(|cx| div().child(self.count.get(cx).to_string()))
//!     }
//! }
//! ```

mod binding;
mod effect;
mod hooks;
mod memo;
mod resource;
mod runtime;
mod signal;
mod window_ext;

pub use binding::Binding;
pub use effect::{Effect, effect, effect_in};
pub use hooks::{ReactiveAppExt, ReactiveContextExt, install};
pub use memo::Memo;
pub use resource::{Resource, ResourceState};
pub use runtime::{EffectId, ReactiveRuntime, SignalId, batch, init, is_installed, track};
pub use signal::{ReadSignal, Signal, WriteSignal};
pub use window_ext::WindowSignalExt;

/// Common relay imports for GPUI views.
pub mod prelude {
    pub use crate::{
        Binding, Effect, Memo, ReactiveAppExt, ReactiveContextExt, Resource, ResourceState, Signal,
        WindowSignalExt, batch, effect, effect_in, init, install, track,
    };
}
