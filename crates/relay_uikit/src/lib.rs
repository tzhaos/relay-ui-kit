//! Relay UIKit for GPUI.
//!
//! This crate groups Relay's GPUI visual tokens, controls, reusable interaction
//! patterns, and gallery binary into one UI package.
//!
//! Components render ordinary GPUI elements and can be wired in two styles:
//!
//! - host-owned state, where the app passes a value snapshot and mutates its
//!   own GPUI entity from event callbacks;
//! - relay-bound state, where form controls receive a [`relay::Binding`] and
//!   update that binding directly for common two-way interactions.
//!
//! The two styles can be mixed during migration. App-specific workflows can
//! keep using GPUI entities and callbacks, while simple fields such as toggles,
//! sliders, selects, and text inputs can move to relay bindings.
//!
//! # Product-grade contract
//!
//! `relay_uikit` is meant to be a production desktop UI toolkit, not a gallery
//! of one-off demos. Exported components and patterns should converge on the
//! same contract:
//!
//! - explicit ownership for value, selection, and open state;
//! - pointer and keyboard parity for actionable controls;
//! - correct focus and dismissal behavior;
//! - accessibility semantics that match the rendered affordance;
//! - resilience under long content, empty states, and host-driven mutations;
//! - tests and gallery scenarios that encode the intended behavior.
//!
//! The long-form version of that contract lives in
//! `docs/relay-uikit-guidelines.md` at the workspace root.

pub(crate) mod component_prelude;
pub mod components;
pub mod interaction;
pub mod patterns;
pub mod styles;

pub use components::{button, choice, controls, display, feedback, form, icon, input, list, row};
pub use patterns::{command, layout, navigation, scroll_surface, shell};
pub use styles::{motion, theme, tone};

pub use button::{Button, ButtonVariant, IconButton};
pub use choice::{Checkbox, Radio, Toggle};
pub use command::{
    CommandPalette, CommandRow, KeybindingActionKind, KeybindingActions, KeybindingRow,
    KeybindingShortcut, KeybindingTable,
};
pub use controls::{
    ColorField, ColorPicker, ColorPreset, ColorSwatch, Disclosure, FilterBar, FilterChip,
    PanelHeader, Segment, SegmentedControl, Slider, Stepper, ThemePreviewCard, ThemePreviewKind,
    ToolbarGroup,
};
pub use display::{
    Badge, BadgeStyle, CountBadge, Divider, EmptyState, Label, LabelColor, LabelSize, StatusDot,
};
pub use feedback::{Banner, Callout, InlineError, LoadingSpinner, ProgressBar, Skeleton, Toast};
pub use form::{FieldDescription, FieldLabel, SettingsRow, SettingsSection};
pub use icon::{Icon, IconName, IconSize, KitAssets};
pub use input::{
    InputActionKind, InputValueKind, NumberInput, NumberInputLayout, SearchField, TextArea,
    TextInput, TextInputAction, TextInputState, ValidationState,
};
pub use list::{
    ForEach, ListItem, ListItemSpacing, SectionedList, SectionedListGroup, TreeNode, TreeView,
};
pub use motion::{MotionDirection, MotionDuration, MotionExt, MotionPolicy};
pub use patterns::overlay::{
    AnchoredOverlay, ConfirmDialog, ContextMenu, Dialog, DropdownMenu, Menu, MenuItem, Overlay,
    Popover, Select, SelectOption, TooltipBody, overlay,
};
pub use patterns::{
    ActionsMenu, CommandMenu, CommandMenuItem, CommandMenuItemKind, DiffHunk, DiffLine,
    DiffLineKind, DiffView, FileKind, FileViewer, InputComposer, ItemPicker, MarkdownViewer,
    OutputLine, OutputLineStyle, OutputLog, OutputSurface, PickerAction, PickerOption, QuickAction,
    SessionRow, SourceView, TabStrip, TabToolbar, TaskRow, TaskRowData,
};
pub use row::{NavRow, TreeRow};
pub use scroll_surface::ScrollSurface;
pub use shell::{
    AppShell, Pane, PaneSurface, PaneToolbar, PaneWidth, SplitAxis, SplitPane, SplitPaneState,
    StatusBar, StatusItem, TitleBar, TopToolbar, WindowControls, WorkspaceBreadcrumb,
};
pub use theme::{ActiveTheme, Theme, radius, space};
pub use tone::Tone;

/// Core controls, tokens, and interaction types.
pub mod core {
    pub use crate::{
        ActiveTheme, Badge, BadgeStyle, Banner, Button, ButtonVariant, Callout, Checkbox,
        ColorField, ColorPicker, ColorPreset, ColorSwatch, CountBadge, Disclosure, Divider,
        EmptyState, FieldDescription, FieldLabel, FilterBar, FilterChip, Icon, IconButton,
        IconName, IconSize, InlineError, InputActionKind, InputValueKind, KitAssets, Label,
        LabelColor, LabelSize, ListItem, ListItemSpacing, LoadingSpinner, MotionDirection,
        MotionDuration, MotionExt, MotionPolicy, NavRow, NumberInput, NumberInputLayout,
        PanelHeader, ProgressBar, Radio, SearchField, SectionedList, SectionedListGroup, Segment,
        SegmentedControl, SettingsRow, SettingsSection, Skeleton, Slider, StatusDot, Stepper,
        TextArea, TextInput, TextInputAction, TextInputState, Theme, ThemePreviewCard,
        ThemePreviewKind, Toast, Toggle, Tone, ToolbarGroup, TreeNode, TreeRow, TreeView,
        ValidationState, button, choice, components, controls, display, feedback, form, icon,
        input, interaction, list, motion, radius, row, space, styles, theme, tone,
    };
}
