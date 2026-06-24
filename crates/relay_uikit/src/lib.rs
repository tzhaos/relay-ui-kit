//! Relay's product UI toolkit for GPUI desktop applications.
//!
//! `relay_uikit` is not a bag of isolated widgets. It is the UI contract that
//! Relay wants application code to build on top of. The crate is organized into
//! three layers:
//!
//! - [`components`]: low-level controls and display primitives such as buttons,
//!   inputs, labels, lists, and feedback surfaces.
//! - [`patterns`]: product-shaped compositions such as overlays, pickers,
//!   command surfaces, output panes, and shell chrome.
//! - [`interaction`]: type-erased adapters that let UIKit stay decoupled from
//!   any particular app entity while still speaking Relay's state vocabulary.
//!
//! # State ownership model
//!
//! UIKit surfaces are expected to make ownership explicit instead of hiding a
//! second state system inside the component.
//!
//! The common modes are:
//!
//! - host-owned snapshot plus callback: the host passes the current value and
//!   updates its own entity or signal in response to events;
//! - Relay-bound control state: the component receives a [`relay::Binding`]
//!   when simple two-way editing is the right model;
//! - Relay selection or open adapters: keyed selection and overlay lifecycle
//!   flow through [`interaction::SelectionSource`],
//!   [`interaction::SelectionBinding`], and [`interaction::OpenState`].
//!
//! These styles are intentionally composable. A workbench can keep workflow
//! state in GPUI entities while letting leaf controls bind directly to Relay
//! primitives for ordinary editing and disclosure behavior.
//!
//! # Product-grade contract
//!
//! Every exported family is expected to converge on the same baseline:
//!
//! - explicit value, selection, and open-state ownership;
//! - pointer and keyboard parity for actionable surfaces;
//! - predictable focus entry, exit, and restoration rules;
//! - accessibility semantics that match the rendered affordance;
//! - resilience under long content, empty data, and host-driven mutation;
//! - tests and gallery scenes that prove behavior under real composition.
//!
//! In other words, the bar is not "renders in the gallery once". The bar is
//! "safe to keep building product surfaces on without replacing the primitive
//! later".
//!
//! # Learning the crate
//!
//! Start with the family modules:
//!
//! - [`components`] for leaf primitives;
//! - [`patterns`] for higher-level compositions;
//! - [`interaction`] for the shared controller vocabulary used across both.
//!
//! The long-form design and review criteria live in the workspace docs:
//!
//! - `docs/relay-uikit-guidelines.md`
//! - `docs/relay-uikit-audit.md`

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
pub use input::{NumberInput, NumberInputLayout, SearchField, TextArea, TextInput, TextInputState};
pub use list::{
    ForEach, ListItem, ListItemSpacing, SectionedList, SectionedListGroup, TreeNode, TreeView,
};
pub use patterns::overlay::{
    AnchoredOverlay, ConfirmDialog, ContextMenu, Dialog, DropdownMenu, Menu, MenuItem, Popover,
    Select, SelectOption, TooltipBody, overlay,
};
pub use patterns::{
    ActionsMenu, CommandMenu, CommandMenuItem, CommandMenuItemKind, DiffHunk, DiffLine,
    DiffLineKind, DiffView, FileKind, FileViewer, InputComposer, ItemPicker, MarkdownViewer,
    OutputLine, OutputLineStyle, OutputLog, OutputSurface, PickerAction, PickerActionKind,
    PickerOption, QuickAction, SessionRow, SourceView, TabStrip, TabToolbar, TaskRow, TaskRowData,
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
        IconName, IconSize, InlineError, KitAssets, Label, LabelColor, LabelSize, ListItem,
        ListItemSpacing, LoadingSpinner, NavRow, NumberInput, NumberInputLayout, PanelHeader,
        ProgressBar, Radio, SearchField, SectionedList, SectionedListGroup, Segment,
        SegmentedControl, SettingsRow, SettingsSection, Skeleton, Slider, StatusDot, Stepper,
        TextArea, TextInput, TextInputState, Theme, ThemePreviewCard, ThemePreviewKind, Toast,
        Toggle, Tone, ToolbarGroup, TreeNode, TreeRow, TreeView, button, choice, components,
        controls, display, feedback, form, icon, input, interaction, list, motion, radius, row,
        space, styles, theme, tone,
    };
}
