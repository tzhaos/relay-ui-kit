//! Picker and action-menu compositions.
//!
//! Picker surfaces sit between basic selection controls and fully custom
//! workbench panels. They package a trigger, a floating panel, keyboard
//! navigation, and optional secondary actions into a reusable contract.
//!
//! Public entry points:
//!
//! - [`ItemPicker`] for keyed selection with optional row actions;
//! - [`ActionsMenu`] for compact action-only menus;
//! - [`PickerOption`], [`PickerAction`], and [`PickerActionKind`] for the
//!   product-facing picker vocabulary.
//!
//! Internals such as `picker_panel` stay module-private because hosts should
//! reason about picker state in terms of selection sources, open state, and
//! callbacks, not panel implementation details.

pub mod actions_menu;
pub mod item_picker;
pub mod picker_panel;
pub mod picker_types;

pub use actions_menu::ActionsMenu;
pub use item_picker::ItemPicker;
pub use picker_types::{PickerAction, PickerActionKind, PickerOption};
