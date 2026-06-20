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

#[cfg(test)]
mod tests {
    #[test]
    fn crate_dependencies_follow_layer_direction() {
        let foundation_cargo = read_crate_toml("foundation");
        let composites_cargo = read_crate_toml("composites");
        let workbench_cargo = read_crate_toml("workbench");
        let gallery_cargo = read_crate_toml("gallery");
        let foundation_components = read_crate_source("foundation", "src/components/mod.rs");
        let foundation_lib = read_crate_source("foundation", "src/lib.rs");
        let composites_lib = read_crate_source("composites", "src/lib.rs");

        // Foundation must not depend on higher layers.
        assert!(!foundation_cargo.contains("relay_composites"));
        assert!(!foundation_cargo.contains("relay_workbench"));
        assert!(!foundation_components.contains(&module_declaration("command")));
        assert!(!foundation_components.contains(&module_declaration("overlay")));
        assert!(!foundation_components.contains("mod select"));
        assert!(!foundation_lib.contains(&module_declaration("structure")));

        // Composites depend on foundation, not on workbench.
        assert!(composites_cargo.contains("relay_foundation.workspace"));
        assert!(!composites_cargo.contains("relay_workbench"));
        assert!(composites_lib.contains(&module_declaration("command")));
        assert!(composites_lib.contains(&module_declaration("overlay")));
        assert!(composites_lib.contains(&module_declaration("scroll_surface")));
        assert!(composites_lib.contains(&module_declaration("select")));

        // Workbench depends on foundation and composites.
        assert!(workbench_cargo.contains("relay_foundation.workspace"));
        assert!(workbench_cargo.contains("relay_composites.workspace"));

        // Gallery is the consumer and depends on workbench.
        assert!(gallery_cargo.contains("relay_workbench.workspace"));
    }

    /// Read a crate's `Cargo.toml` at test time using `CARGO_MANIFEST_DIR`.
    fn read_crate_toml(crate_dir: &str) -> String {
        read_crate_source(crate_dir, "Cargo.toml")
    }

    fn read_crate_source(crate_dir: &str, relative_path: &str) -> String {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let Some(workspace_root) = manifest_dir.parent().and_then(|p| p.parent()) else {
            panic!("workspace root");
        };
        let path = workspace_root
            .join("crates")
            .join(crate_dir)
            .join(relative_path);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
    }

    fn module_declaration(name: &str) -> String {
        format!("pub mod {name}")
    }
}
