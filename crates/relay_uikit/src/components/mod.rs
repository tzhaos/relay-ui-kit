//! Product-grade leaf components for Relay desktop surfaces.
//!
//! This layer owns the reusable building blocks that higher-level patterns are
//! allowed to depend on: buttons, choices, inputs, lists, display primitives,
//! and feedback surfaces.
//!
//! Components in this module should solve local UI concerns well:
//!
//! - paint, sizing, and spacing;
//! - pointer and keyboard interaction;
//! - focus affordance and tabbability;
//! - accessibility semantics the host cannot infer for itself;
//! - resilience under long labels, empty values, and disabled state.
//!
//! They should *not* own broader workflow state such as:
//!
//! - which dialog is open;
//! - which resource is loading;
//! - how a workbench pane is orchestrated.
//!
//! That separation is what keeps [`crate::patterns`] composable instead of
//! forcing product code to tunnel through bespoke component-local controllers.
//!
//! Most families in this layer intentionally support either host-owned snapshot
//! usage, Relay-bound usage, or both. The relevant module docs call out which
//! ownership style is expected for each family.

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
