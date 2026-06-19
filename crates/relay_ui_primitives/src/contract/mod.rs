//! Code-level UI contract for Relay's component system.
//!
//! This module is not a component layer. It names the state, input, event,
//! motion, layout, and composition rules that implementations must follow.

mod composition;
mod event;
mod input;
mod layout;
mod motion;
mod state;

pub use composition::{LAYER_DEPENDENCIES, Layer, LayerDependency};
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
