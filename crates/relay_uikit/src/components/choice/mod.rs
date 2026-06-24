//! Boolean and keyed choice controls.
//!
//! This family covers compact "choose or toggle something" primitives:
//!
//! - [`Checkbox`] for independent boolean flags;
//! - [`Toggle`] for switch-like binary state;
//! - [`Radio`] for one-of-many keyed selection.
//!
//! These controls are designed to work in both of Relay's common ownership
//! styles:
//!
//! - direct host snapshots plus callbacks for product-specific workflows;
//! - Relay bindings or selection adapters for ordinary form state.
//!
//! Product expectations for this family:
//!
//! - click, `Enter`, and `Space` activate the same state transition;
//! - disabled state blocks both pointer and keyboard paths;
//! - selected/toggled semantics are exposed instead of only implied visually;
//! - dense settings and toolbar compositions remain readable and stable.

mod checkbox;
mod radio;
mod toggle;

pub use checkbox::Checkbox;
pub use radio::Radio;
pub use toggle::Toggle;
