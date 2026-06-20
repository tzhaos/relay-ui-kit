//! Relay foundation layer for GPUI.
//!
//! This crate owns the visual tokens, icon system, input models, and generic
//! controls that do not know about Relay workbench concepts such as terminals,
//! agents, projects, branches, or viewers.

pub(crate) mod component_prelude;
pub mod components;
pub mod interaction;
pub mod styles;

pub use components::{button, choice, controls, display, feedback, form, icon, input, list, row};
pub use styles::{motion, theme, tone};

pub use button::{Button, ButtonVariant, IconButton};
pub use choice::{Checkbox, Radio, Toggle};
pub use controls::{
    ColorField, ColorPicker, ColorPreset, ColorSwatch, Disclosure, FilterBar, FilterChip,
    PanelHeader, SearchField, Segment, SegmentedControl, Slider, Stepper, ThemePreviewCard,
    ThemePreviewKind, ToolbarGroup,
};
pub use display::{
    Badge, BadgeStyle, CountBadge, Divider, EmptyState, Label, LabelColor, LabelSize, StatusDot,
};
pub use feedback::{Banner, Callout, InlineError, LoadingSpinner, ProgressBar, Skeleton, Toast};
pub use form::{FieldDescription, FieldLabel, SettingsRow, SettingsSection};
pub use icon::{Icon, IconName, IconSize, KitAssets};
pub use input::{
    InputActionKind, InputValueKind, NumberInput, NumberInputLayout, TextArea, TextInput,
    TextInputAction, TextInputState, ValidationState,
};
pub use list::{ListItem, ListItemSpacing, SectionedList, SectionedListGroup, TreeNode, TreeView};
pub use motion::{MotionDirection, MotionDuration, MotionExt, MotionPolicy};
pub use row::{NavRow, TaskRow, TaskRowData, TreeRow};
pub use theme::{ActiveTheme, Theme, radius, space};
pub use tone::Tone;
