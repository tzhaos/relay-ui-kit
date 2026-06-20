//! Foundational GPUI primitives for Relay UI.
//!
//! This crate owns the visual tokens, icon system, input models, and generic
//! controls that do not know about Relay workbench concepts such as terminals,
//! agents, projects, branches, or viewers.

pub(crate) mod component_prelude;
pub mod components;
pub mod contract;
pub mod interaction;
pub mod structure;
pub mod styles;

pub use components::{
    button, choice, command, controls, display, feedback, form, icon, input, list, row,
};
pub use structure::ScrollSurface;
pub use styles::{motion, theme, tone};

pub use button::{Button, ButtonVariant, IconButton};
pub use choice::{Checkbox, Radio, Toggle};
pub use command::{
    CommandPalette, CommandRow, KeybindingActionKind, KeybindingActions, KeybindingRow,
    KeybindingTable, KeybindingShortcut, KeyboardShortcut,
};
pub use components::overlay::{
    ConfirmDialog, ContextMenu, Dialog, DropdownMenu, Menu, MenuItem, Overlay, Popover,
    TooltipBody, overlay,
};
pub use controls::{
    ColorField, ColorPicker, ColorPreset, ColorSwatch, Disclosure, FilterBar, FilterChip,
    PanelHeader, SearchField, Segment, SegmentedControl, Select, SelectOption, Slider, Stepper,
    ThemePreviewCard, ThemePreviewKind, ToolbarGroup,
};
pub use display::{
    Badge, BadgeStyle, CountBadge, Divider, EmptyState, Label, LabelColor, LabelSize, StatusDot,
};
pub use feedback::{Banner, Callout, InlineError, LoadingSpinner, ProgressBar, Skeleton, Toast};
pub use form::{FieldDescription, FieldLabel, SettingsRow, SettingsSection};
pub use icon::{Icon, IconName, IconSize, KitAssets};
pub use input::{
    NumberInput, NumberInputLayout, TextArea, TextInput, TextInputAction, TextInputState,
};
pub use list::{ListItem, ListItemSpacing, SectionedList, SectionedListGroup, TreeNode, TreeView};
pub use motion::MotionExt;
pub use row::{NavRow, TaskRow, TaskRowData, TreeRow};
pub use theme::{ActiveTheme, Theme, radius, space};
pub use tone::Tone;
