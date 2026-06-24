//! Reusable controls for settings, tool panels, and compact desktop surfaces.
//!
//! This module groups controls that are more structured than a single action,
//! but still remain leaf-level UI rather than full product patterns. Common
//! examples include:
//!
//! - compact setting controls such as [`Slider`], [`Stepper`], and
//!   [`Disclosure`];
//! - keyed selection controls such as [`SegmentedControl`] and [`FilterChip`];
//! - presentational helpers such as [`PanelHeader`] and [`ToolbarGroup`];
//! - theme and color controls used in settings and appearance flows.
//!
//! The contract for this family is intentionally strict:
//!
//! - state ownership must stay obvious;
//! - keyboard behavior must match desktop expectations;
//! - controls must survive constrained sidebars and dense forms;
//! - no control should invent app-specific workflow state of its own.

mod color_picker;
mod disclosure;
mod filter;
mod panel_header;
mod segmented_control;
mod slider;
mod stepper;
mod swatch;
mod theme_preview;
mod toolbar_group;

pub use color_picker::{ColorPicker, ColorPreset};
pub use disclosure::Disclosure;
pub use filter::{FilterBar, FilterChip};
pub use panel_header::PanelHeader;
pub use segmented_control::{Segment, SegmentedControl};
pub use slider::Slider;
pub use stepper::Stepper;
pub use swatch::{ColorField, ColorSwatch};
pub use theme_preview::{ThemePreviewCard, ThemePreviewKind};
pub use toolbar_group::ToolbarGroup;
