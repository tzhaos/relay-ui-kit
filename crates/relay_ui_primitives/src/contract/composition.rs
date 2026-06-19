#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Primitive,
    Component,
    Workbench,
    Gallery,
}

impl Layer {
    pub fn may_depend_on(self, dependency: Self) -> bool {
        matches!(
            (self, dependency),
            (Layer::Primitive, Layer::Primitive)
                | (Layer::Component, Layer::Primitive | Layer::Component)
                | (
                    Layer::Workbench,
                    Layer::Primitive | Layer::Component | Layer::Workbench
                )
                | (
                    Layer::Gallery,
                    Layer::Primitive | Layer::Component | Layer::Workbench | Layer::Gallery
                )
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Read a crate's `Cargo.toml` at test time using `CARGO_MANIFEST_DIR`.
    /// This avoids fragile relative `include_str!` paths that break when source
    /// files are moved.
    fn read_cargo_toml(crate_name: &str) -> String {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .expect("workspace root");
        let path = workspace_root
            .join("crates")
            .join(crate_name)
            .join("Cargo.toml");
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
    }

    #[test]
    fn primitive_layer_only_depends_on_itself() {
        assert!(Layer::Primitive.may_depend_on(Layer::Primitive));
    }

    #[test]
    fn primitive_layer_cannot_depend_on_higher_layers() {
        assert!(!Layer::Primitive.may_depend_on(Layer::Component));
    }

    #[test]
    fn component_layer_can_depend_on_primitives() {
        assert!(Layer::Component.may_depend_on(Layer::Primitive));
    }

    #[test]
    fn component_layer_cannot_depend_on_workbench() {
        assert!(!Layer::Component.may_depend_on(Layer::Workbench));
    }

    #[test]
    fn cargo_dependencies_follow_layer_direction() {
        let primitives_cargo = read_cargo_toml("relay_ui_primitives");
        let components_cargo = read_cargo_toml("relay_ui_components");
        let workbench_cargo = read_cargo_toml("relay_workbench_ui");
        let gallery_cargo = read_cargo_toml("relay_gallery");

        assert!(!primitives_cargo.contains("relay_ui_components"));
        assert!(!primitives_cargo.contains("relay_workbench_ui"));
        assert!(components_cargo.contains("relay_ui_primitives.workspace"));
        assert!(!components_cargo.contains("relay_workbench_ui"));
        assert!(workbench_cargo.contains("relay_ui_primitives.workspace"));
        assert!(workbench_cargo.contains("relay_ui_components.workspace"));
        assert!(gallery_cargo.contains("relay_workbench_ui.workspace"));
    }
}
