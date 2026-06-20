//! Relay patterns layer for GPUI layout components.
//!
//! This layer depends on core and provides reusable shell, structure,
//! command, overlay, scroll, tab, and title-bar patterns without terminal or
//! agent product semantics.

pub mod command;
pub mod layout;
pub mod overlay;
pub mod scroll_surface;

pub use command::{
    CommandPalette, CommandRow, KeybindingActionKind, KeybindingActions, KeybindingRow,
    KeybindingShortcut, KeybindingTable,
};
pub use layout::{shell, structure};
pub use overlay::{
    ConfirmDialog, ContextMenu, Dialog, DropdownMenu, Menu, MenuItem, Overlay, Popover,
    TooltipBody, overlay,
};
pub use scroll_surface::ScrollSurface;
pub use shell::{
    AppShell, Pane, PaneSurface, PaneToolbar, PaneWidth, SplitAxis, SplitPane, SplitPaneState,
    StatusBar, StatusItem, TitleBar, TopToolbar, WindowControls, WorkspaceBreadcrumb,
};
pub use structure::{KeyValue, ListSection, Tab, Tabs};
