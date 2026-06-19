//! Relay UI kit — a GPUI component library in the Orca product direction.
//!
//! Every component here is a `RenderOnce` builder that reads the active
//! [`theme::Theme`] from the [`gpui::App`] globals and carries generic click /
//! key callbacks. None of them depend on a concrete view, so the same component
//! drops into the gallery, the real workbench, or a test without dragging app
//! state into scope.
//!
//! Stateful controls (text input, checkbox, dropdown open/closed) follow a
//! model/view split: the host owns a small state struct (e.g.
//! [`input::TextInputState`]) and feeds events into it, while the component is a
//! stateless renderer of that state. This keeps the library free of hidden
//! global state and lets the host drive everything.
//!
//! Install the theme once at startup with [`theme::init`], and register the
//! embedded icon set with `Application::new().with_assets(icon::KitAssets)`.

pub(crate) mod component_prelude;
pub mod components;
pub(crate) mod interaction;
pub mod layout;
pub mod patterns;
pub mod prelude;
pub mod styles;

pub use components::{
    button, choice, command, controls, display, feedback, form, icon, input, list, row,
};
pub use layout::{shell, structure};
pub use patterns::{composer, git, launcher, terminal, viewer};
pub use styles::{motion, theme, tone};

// Flat re-exports so callers write `relay_ui_kit::Button` etc.
pub use button::{Button, ButtonVariant, IconButton};
pub use choice::{Checkbox, Radio, Toggle};
pub use command::{CommandPalette, CommandRow, KeybindingRow, KeybindingTable, KeyboardShortcut};
pub use components::overlay::{
    ConfirmDialog, ContextMenu, Dialog, DropdownMenu, Menu, MenuItem, Overlay, Popover,
    TooltipBody, overlay,
};
pub use composer::Composer;
pub use controls::{
    ColorField, ColorSwatch, PanelHeader, SearchField, Segment, SegmentedControl, Select,
    SelectOption, Slider,
};
pub use display::{Badge, BadgeStyle, Divider, EmptyState, StatusDot};
pub use feedback::{Banner, InlineError, LoadingSpinner, ProgressBar, Skeleton, Toast};
pub use form::{FieldDescription, FieldLabel, SettingsRow, SettingsSection};
pub use git::{
    BranchActionKind, BranchActionsMenu, BranchOption, BranchPickerAction, BranchSelector,
};
pub use icon::{Icon, IconName, IconSize, KitAssets};
pub use input::{NumberInput, TextArea, TextInput, TextInputAction, TextInputState};
pub use launcher::{LauncherItem, LauncherItemKind, LauncherMenu};
pub use list::{ListItem, ListItemSpacing, SectionedList, SectionedListGroup, TreeNode, TreeView};
pub use motion::{MotionDirection, MotionDuration, MotionExt};
pub use row::{NavRow, TaskRow, TaskRowData, TreeRow};
pub use shell::{
    AppShell, Pane, PaneSurface, PaneToolbar, PaneWidth, SplitAxis, SplitPane, SplitPaneState,
    StatusBar, StatusItem, TitleBar, TopToolbar, WindowControls, WorkspaceBreadcrumb,
};
pub use structure::{KeyValue, ListSection, ScrollSurface, Tab, Tabs};
pub use terminal::{
    AgentQuickLaunch, TerminalLine, TerminalLineStyle, TerminalSessionRow, TerminalStatusBadge,
    TerminalSurface, TerminalTab, TerminalToolbar,
};
pub use theme::{ActiveTheme, Theme, radius, space};
pub use tone::Tone;
pub use viewer::{
    CodeView, DiffHunk, DiffLine, DiffLineKind, DiffView, FileKind, FileView, MarkdownView,
};
