//! Relay patterns layer for GPUI components.
//!
//! This layer depends on core and provides reusable command, display, layout,
//! navigation, overlay, scroll, and composite patterns.
//!
//! Patterns are where component correctness is tested against realistic product
//! composition: selection churn, overlay lifecycle, async surfaces, retained row
//! identity, and multi-pane workbench layouts. They should not reintroduce a
//! second state model; instead they adapt Relay bindings, open state, and
//! selection sources into reusable UI assemblies.

pub mod command;
pub mod display;
pub mod layout;
pub mod navigation;
pub mod overlay;
pub mod scroll_surface;

// Composite patterns (migrated from workbench with generic names)
pub mod command_menu;
pub mod diff_view;
pub mod file_viewer;
pub mod input_composer;
pub mod markdown_viewer;
pub mod output_line;
pub mod output_log;
pub mod output_resource;
pub mod output_surface;
pub mod picker;
pub mod quick_action;
pub mod session_row;
pub mod source_view;
pub mod tab_strip;
pub mod tab_toolbar;
pub mod task_row;

pub use command::{
    CommandPalette, CommandRow, KeybindingActionKind, KeybindingActions, KeybindingRow,
    KeybindingShortcut, KeybindingTable,
};
pub use layout::shell;
pub use overlay::{
    AnchoredOverlay, ConfirmDialog, ContextMenu, Dialog, DropdownMenu, Menu, MenuItem, Popover,
    Select, SelectOption, TooltipBody, overlay,
};
pub use scroll_surface::ScrollSurface;
pub use shell::{
    AppShell, Pane, PaneSurface, PaneToolbar, PaneWidth, SplitAxis, SplitPane, SplitPaneState,
    StatusBar, StatusItem, TitleBar, TopToolbar, WindowControls, WorkspaceBreadcrumb,
};

// Composite pattern exports
pub use command_menu::{CommandMenu, CommandMenuItem, CommandMenuItemKind};
pub use diff_view::{DiffHunk, DiffLine, DiffLineKind, DiffView};
pub use file_viewer::{FileKind, FileViewer};
pub use input_composer::InputComposer;
pub use markdown_viewer::MarkdownViewer;
pub use output_line::{OutputLine, OutputLineStyle};
pub use output_log::OutputLog;
pub use output_resource::{OutputResourceSnapshot, output_resource_snapshot};
pub use output_surface::OutputSurface;
pub use picker::actions_menu::ActionsMenu;
pub use picker::{ItemPicker, PickerAction, PickerActionKind, PickerOption};
pub use quick_action::QuickAction;
pub use session_row::SessionRow;
pub use source_view::SourceView;
pub use tab_strip::TabStrip;
pub use tab_toolbar::TabToolbar;
pub use task_row::{TaskRow, TaskRowData};
