//! Code-level UI contract for Relay's component system.
//!
//! This module is **not** a component layer.  It names the state, input, event,
//! motion, layout, and composition rules that all implementations must follow.
//!
//! # Submodules
//!
//! | Module | Governs |
//! |---|---|
//! | [`composition`] | Crate dependency layering (Primitive → Component → Workbench → Gallery) |
//! | [`state`] | Which components use `HostOwned` vs `WindowKeyed` state |
//! | [`motion`] | Animation duration, direction, and policy per component |
//! | [`event`] | Standard event handler names (`on_click`, `on_select`, etc.) |
//! | [`input`] | Input value kinds, action kinds, and validation states |
//! | [`layout`] | Border widths, radius scale, scrollbar dimensions, overlay priorities |
//!
//! # How rules are enforced
//!
//! - **Composition rules** are verified at test time by parsing each crate's
//!   `Cargo.toml` and checking that no crate depends on a higher layer.
//! - **State, motion, input, and event rules** are validated by inline tests
//!   in their respective submodules.  Additional enforcement relies on code
//!   review convention.
//! - **Layout constants** are used throughout the codebase; any deviation is
//!   caught by visual review in the gallery app.
//!
//! # Adding a new component
//!
//! When adding a component to `relay_ui_primitives`:
//!
//! 1. Add a [`StateRule`] in [`state`] declaring its state ownership model.
//! 2. If the component animates, add a [`MotionRule`] in [`motion`].
//! 3. Follow the handler naming conventions in [`event`] for callback methods.
//! 4. Use the layout constants from [`layout`] for borders, radii, and spacing.

mod composition;
mod event;
mod input;
mod layout;
mod motion;
mod state;

pub use composition::Layer;
pub use event::EventKind;
pub use input::{InputActionKind, InputValueKind, ValidationState};
pub use layout::{
    BORDER_WIDTH, BorderRule, OVERLAY_PRIORITY_DIALOG, OVERLAY_PRIORITY_FLOATING,
    OVERLAY_WINDOW_MARGIN, OverlayLayer, RADIUS_LG, RADIUS_MD, RADIUS_SM, SCROLL_GUTTER_WIDTH,
    SCROLL_MIN_THUMB_HEIGHT, SCROLL_THUMB_WIDTH, ShadowRule,
};
pub use motion::{
    MOTION_RULES, MotionDirection, MotionDisablePolicy, MotionDuration, MotionPolicy, MotionRule,
};
pub use state::{STATE_RULES, StateOwnership, StateRule, state_rule};
