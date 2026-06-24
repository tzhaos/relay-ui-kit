//! Layout primitives for workbench-scale desktop composition.
//!
//! This layer provides reusable structure rather than app-specific workflow:
//!
//! - [`ListSection`] for titled grouped content blocks;
//! - [`shell`] for panes, split layout, toolbars, title bars, and status bars.
//!
//! Layout primitives should make geometry ownership explicit. In particular,
//! shell-level components should separate:
//!
//! - durable layout state such as split sizes and pane widths;
//! - leaf interaction state such as button hover or text selection;
//! - host workflow state such as which resource is loading or which editor is active.

mod list_section;
pub mod shell;

pub use list_section::ListSection;
pub use shell::*;
