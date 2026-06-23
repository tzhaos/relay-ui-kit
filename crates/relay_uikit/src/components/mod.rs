//! Core reusable components with no Relay product workflow state.
//!
//! These are the base primitives that higher-level patterns depend on.
//! A component in this layer is expected to be product-grade before it is used
//! as a foundation elsewhere, which means:
//!
//! - its ownership model is explicit;
//! - keyboard and pointer interaction are intentionally defined;
//! - accessibility semantics are present where the host cannot infer them;
//! - constrained layout and disabled-state behavior are treated as first-class.

pub mod button;
pub(crate) mod button_like;
pub mod choice;
pub mod controls;
pub mod display;
pub mod feedback;
pub mod form;
pub mod icon;
pub mod input;
pub mod list;
pub mod row;

pub use button::*;
pub use choice::*;
pub use controls::*;
pub use display::*;
pub use feedback::*;
pub use form::*;
pub use icon::*;
pub use input::*;
pub use list::*;
pub use row::*;
