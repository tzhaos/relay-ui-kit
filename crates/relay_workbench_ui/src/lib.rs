//! Relay workbench UI compositions.
//!
//! This layer is allowed to speak in Relay product terms such as terminals,
//! agents, git branches, launchers, and read-only file viewers.

pub mod workbench;

pub use relay_ui_components::{layout, shell, structure};
pub use relay_ui_primitives::{
    ActiveTheme, Badge, BadgeStyle, Banner, Button, ButtonVariant, Callout, Checkbox, ColorField,
    ColorPicker, ColorPreset, ColorSwatch, CommandPalette, CommandRow, ConfirmDialog, ContextMenu,
    CountBadge, Dialog, Disclosure, Divider, DropdownMenu, EmptyState, FieldDescription,
    FieldLabel, FilterBar, FilterChip, Icon, IconButton, IconName, IconSize, InlineError,
    KeybindingActionKind, KeybindingActions, KeybindingRow, KeybindingTable, KeyboardShortcut,
    KitAssets, Label, LabelColor, LabelSize, ListItem, ListItemSpacing, LoadingSpinner, Menu,
    MenuItem, MotionDirection, MotionDuration, MotionExt, NavRow, NumberInput, NumberInputLayout,
    Overlay, PanelHeader, Popover, ProgressBar, Radio, SearchField, SectionedList,
    SectionedListGroup, Segment, SegmentedControl, Select, SelectOption, SettingsRow,
    SettingsSection, Skeleton, Slider, StatusDot, Stepper, TaskRow, TaskRowData, TextArea,
    TextInput, TextInputAction, TextInputState, Theme, ThemePreviewCard, ThemePreviewKind, Toast,
    Toggle, Tone, ToolbarGroup, TooltipBody, TreeNode, TreeRow, TreeView, button, choice, command,
    components, controls, display, feedback, form, icon, input, interaction, list, motion, overlay,
    radius, row, space, styles, theme, tone,
};
pub use workbench::{composer, git, launcher, terminal, viewer};

pub use composer::Composer;
pub use git::{
    BranchActionKind, BranchActionsMenu, BranchOption, BranchPickerAction, BranchSelector,
};
pub use launcher::{LauncherItem, LauncherItemKind, LauncherMenu};
pub use terminal::{
    AgentQuickLaunch, TerminalLine, TerminalLineStyle, TerminalSessionRow, TerminalStatusBadge,
    TerminalSurface, TerminalTab, TerminalToolbar, TerminalTranscript,
};
pub use viewer::{
    CodeView, DiffHunk, DiffLine, DiffLineKind, DiffView, FileKind, FileView, MarkdownView,
};
