//! Command and launcher primitives.
//!
//! This layer is generic: it renders command palette rows and shortcuts, but it
//! does not know whether a command opens a terminal, launches an agent, or
//! changes workbench focus.

mod command_palette;
mod command_row;
mod keybinding_actions;
mod keybinding_table;
mod keyboard_shortcut;

pub use command_palette::CommandPalette;
pub use command_row::CommandRow;
pub use keybinding_actions::{KeybindingActionKind, KeybindingActions};
pub use keybinding_table::{KeybindingRow, KeybindingTable};
pub use keyboard_shortcut::KeybindingShortcut;
#[deprecated(note = "use KeybindingShortcut")]
pub type KeyboardShortcut = KeybindingShortcut;
