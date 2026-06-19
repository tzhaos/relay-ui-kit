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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerDependency {
    pub layer: Layer,
    pub may_depend_on: &'static [Layer],
}

pub const LAYER_DEPENDENCIES: &[LayerDependency] = &[
    LayerDependency {
        layer: Layer::Primitive,
        may_depend_on: &[Layer::Primitive],
    },
    LayerDependency {
        layer: Layer::Component,
        may_depend_on: &[Layer::Primitive, Layer::Component],
    },
    LayerDependency {
        layer: Layer::Workbench,
        may_depend_on: &[Layer::Primitive, Layer::Component, Layer::Workbench],
    },
    LayerDependency {
        layer: Layer::Gallery,
        may_depend_on: &[
            Layer::Primitive,
            Layer::Component,
            Layer::Workbench,
            Layer::Gallery,
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    const PRIMITIVES_CARGO: &str = include_str!("../../Cargo.toml");
    const COMPONENTS_CARGO: &str = include_str!("../../../relay_ui_components/Cargo.toml");
    const WORKBENCH_CARGO: &str = include_str!("../../../relay_workbench_ui/Cargo.toml");
    const GALLERY_CARGO: &str = include_str!("../../../relay_gallery/Cargo.toml");

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
        assert!(!PRIMITIVES_CARGO.contains("relay_ui_components"));
        assert!(!PRIMITIVES_CARGO.contains("relay_workbench_ui"));
        assert!(COMPONENTS_CARGO.contains("relay_ui_primitives.workspace"));
        assert!(!COMPONENTS_CARGO.contains("relay_workbench_ui"));
        assert!(WORKBENCH_CARGO.contains("relay_ui_primitives.workspace"));
        assert!(WORKBENCH_CARGO.contains("relay_ui_components.workspace"));
        assert!(GALLERY_CARGO.contains("relay_workbench_ui.workspace"));
    }
}
