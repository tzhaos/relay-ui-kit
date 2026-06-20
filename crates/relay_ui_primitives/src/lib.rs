//! Foundational GPUI primitives for Relay UI.
//!
//! This crate owns the visual tokens, icon system, input models, and generic
//! controls that do not know about Relay workbench concepts such as terminals,
//! agents, projects, branches, or viewers.

pub(crate) mod component_prelude;
pub mod components;
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
#[allow(deprecated)]
pub use command::KeyboardShortcut;
pub use command::{
    CommandPalette, CommandRow, KeybindingActionKind, KeybindingActions, KeybindingRow,
    KeybindingShortcut, KeybindingTable,
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
    InputActionKind, InputValueKind, NumberInput, NumberInputLayout, TextArea, TextInput,
    TextInputAction, TextInputState, ValidationState,
};
pub use list::{ListItem, ListItemSpacing, SectionedList, SectionedListGroup, TreeNode, TreeView};
pub use motion::{MotionDirection, MotionDuration, MotionExt, MotionPolicy};
pub use row::{NavRow, TaskRow, TaskRowData, TreeRow};
pub use theme::{ActiveTheme, Theme, radius, space};
pub use tone::Tone;

#[cfg(test)]
mod tests {
    #[test]
    fn crate_dependencies_follow_layer_direction() {
        let primitives_cargo = read_crate_toml("relay_ui_primitives");
        let components_cargo = read_crate_toml("relay_ui_components");
        let workbench_cargo = read_crate_toml("relay_workbench_ui");
        let gallery_cargo = read_crate_toml("relay_gallery");

        // Primitives must not depend on higher layers
        assert!(!primitives_cargo.contains("relay_ui_components"));
        assert!(!primitives_cargo.contains("relay_workbench_ui"));

        // Components: depends on primitives, not on workbench
        assert!(components_cargo.contains("relay_ui_primitives.workspace"));
        assert!(!components_cargo.contains("relay_workbench_ui"));

        // Workbench: depends on primitives and components
        assert!(workbench_cargo.contains("relay_ui_primitives.workspace"));
        assert!(workbench_cargo.contains("relay_ui_components.workspace"));

        // Gallery: the consumer — depends on workbench
        assert!(gallery_cargo.contains("relay_workbench_ui.workspace"));
    }

    /// Read a crate's `Cargo.toml` at test time using `CARGO_MANIFEST_DIR`.
    fn read_crate_toml(crate_name: &str) -> String {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let Some(workspace_root) = manifest_dir.parent().and_then(|p| p.parent()) else {
            panic!("workspace root");
        };
        let path = workspace_root
            .join("crates")
            .join(crate_name)
            .join("Cargo.toml");
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
    }
}
