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

pub mod components;
pub mod layout;
pub mod patterns;
pub mod prelude;
pub mod styles;

#[path = "components/button.rs"]
pub mod button;
#[path = "components/choice.rs"]
pub mod choice;
#[path = "components/command/mod.rs"]
pub mod command;
#[path = "components/controls/mod.rs"]
pub mod controls;
#[path = "components/display.rs"]
pub mod display;
#[path = "components/feedback/mod.rs"]
pub mod feedback;
#[path = "components/form/mod.rs"]
pub mod form;
#[path = "patterns/git/mod.rs"]
pub mod git;
#[path = "components/icon.rs"]
pub mod icon;
#[path = "components/input/mod.rs"]
pub mod input;
#[path = "patterns/launcher.rs"]
pub mod launcher;
#[path = "styles/motion.rs"]
pub mod motion;
#[path = "components/overlay/mod.rs"]
pub mod overlay;
#[path = "components/row/mod.rs"]
pub mod row;
#[path = "layout/shell/mod.rs"]
pub mod shell;
#[path = "layout/structure/mod.rs"]
pub mod structure;
#[path = "patterns/terminal/mod.rs"]
pub mod terminal;
#[path = "styles/theme.rs"]
pub mod theme;
#[path = "styles/tone.rs"]
pub mod tone;
#[path = "patterns/viewer/mod.rs"]
pub mod viewer;

// Flat re-exports so callers write `relay_ui_kit::Button` etc.
pub use button::{Button, ButtonVariant, IconButton};
pub use choice::{Checkbox, Radio, Toggle};
pub use command::{CommandPalette, CommandRow, KeyboardShortcut};
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
pub use input::{NumberInput, TextInput, TextInputAction, TextInputState};
pub use launcher::{LauncherItem, LauncherItemKind, LauncherMenu};
pub use motion::{MotionDirection, MotionDuration, MotionExt};
pub use overlay::{Menu, MenuItem, Overlay, TooltipBody, overlay};
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
