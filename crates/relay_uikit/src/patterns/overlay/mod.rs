//! Overlay, menu, and dialog primitives with a shared controller vocabulary.
//!
//! This module groups every surface that temporarily floats above ordinary
//! layout:
//!
//! - anchored menus and dropdowns;
//! - context menus and lightweight popovers;
//! - dialogs and confirm flows;
//! - select-style trigger plus floating choice surfaces.
//!
//! Product-grade overlays in Relay are expected to share the same baseline:
//!
//! - host-controlled or Relay-controlled open state rather than hidden booleans;
//! - explicit dismissal behavior for outside click and `Escape`;
//! - predictable focus entry when the surface opens;
//! - predictable focus restoration when it closes;
//! - no stale anchor, stale submenu, or stuck-open behavior under repeated churn.
//!
//! Prefer the more specific surface when possible:
//!
//! - [`Popover`] for lightweight inline detail;
//! - [`Menu`] or [`DropdownMenu`] for command lists;
//! - [`Select`] and [`crate::patterns::ItemPicker`] for controlled selection;
//! - [`Dialog`] and [`ConfirmDialog`] for blocking decisions and workflow pivots.

mod anchored_overlay;
mod context_menu;
mod dialog;
mod dropdown_menu;
mod floating_overlay;
mod menu;
mod popover;
mod select;
mod tooltip_body;

pub use anchored_overlay::AnchoredOverlay;
pub use context_menu::ContextMenu;
pub use dialog::{ConfirmDialog, Dialog};
pub use dropdown_menu::DropdownMenu;
pub use floating_overlay::overlay;
pub use menu::{Menu, MenuItem};
pub use popover::Popover;
pub use select::{Select, SelectOption};
pub use tooltip_body::TooltipBody;
