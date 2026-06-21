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
mod context;
mod effect;
mod form;
mod hooks;
mod keyed;
mod memo;
mod resource;
mod runtime;
mod signal;
mod signal_vec;
pub mod view;
mod window_ext;

pub use binding::Binding;
pub use context::{ContextHandle, provide_context, use_context};
pub use effect::{Effect, effect, effect_in};
pub use form::Form;
pub use hooks::{ReactiveAppExt, ReactiveContextExt, install};
pub use keyed::{KeyedSubView, KeyedSubViews};
pub use memo::Memo;
pub use relay_macros::Reactive;
pub use resource::{Resource, ResourceState};
pub use runtime::{EffectId, ReactiveRuntime, SignalId, batch, init, is_installed, track, untrack};
pub use signal::{ReadSignal, Signal, WriteSignal};
pub use signal_vec::SignalVecExt;
pub use view::{FormBuilder, ReactiveView, StateScope, SubView};
pub use window_ext::WindowSignalExt;

/// Common relay imports for GPUI views.
pub mod prelude {
    pub use crate::view;
    pub use crate::{
        Binding, ContextHandle, Effect, Form, FormBuilder, KeyedSubView, KeyedSubViews, Memo,
        Reactive, ReactiveAppExt, ReactiveContextExt, ReactiveView, Resource, ResourceState,
        Signal, SignalVecExt, StateScope, SubView, WindowSignalExt, batch, effect, effect_in, init,
        install, provide_context, track, untrack, use_context,
    };
}
