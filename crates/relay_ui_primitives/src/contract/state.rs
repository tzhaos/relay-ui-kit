use super::Layer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateOwnership {
    HostOwned,
    WindowKeyed,
    VisualLocal,
}

impl StateOwnership {
    pub fn is_semantic_host_state(self) -> bool {
        matches!(self, Self::HostOwned)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateRule {
    pub component: &'static str,
    pub layer: Layer,
    pub ownership: StateOwnership,
}

pub const STATE_RULES: &[StateRule] = &[
    StateRule {
        component: "TextInput",
        layer: Layer::Primitive,
        ownership: StateOwnership::HostOwned,
    },
    StateRule {
        component: "TextArea",
        layer: Layer::Primitive,
        ownership: StateOwnership::HostOwned,
    },
    StateRule {
        component: "NumberInput",
        layer: Layer::Primitive,
        ownership: StateOwnership::HostOwned,
    },
    StateRule {
        component: "Select",
        layer: Layer::Primitive,
        ownership: StateOwnership::HostOwned,
    },
    StateRule {
        component: "DropdownMenu",
        layer: Layer::Primitive,
        ownership: StateOwnership::HostOwned,
    },
    StateRule {
        component: "SegmentedControl",
        layer: Layer::Primitive,
        ownership: StateOwnership::HostOwned,
    },
    StateRule {
        component: "Checkbox",
        layer: Layer::Primitive,
        ownership: StateOwnership::HostOwned,
    },
    StateRule {
        component: "Toggle",
        layer: Layer::Primitive,
        ownership: StateOwnership::HostOwned,
    },
    StateRule {
        component: "TreeView",
        layer: Layer::Primitive,
        ownership: StateOwnership::HostOwned,
    },
    StateRule {
        component: "ScrollSurface",
        layer: Layer::Primitive,
        ownership: StateOwnership::WindowKeyed,
    },
    StateRule {
        component: "SplitPane",
        layer: Layer::Component,
        ownership: StateOwnership::HostOwned,
    },
    StateRule {
        component: "BranchSelector",
        layer: Layer::Workbench,
        ownership: StateOwnership::HostOwned,
    },
    StateRule {
        component: "LauncherMenu",
        layer: Layer::Workbench,
        ownership: StateOwnership::HostOwned,
    },
];

pub fn state_rule(component: &str) -> Option<&'static StateRule> {
    STATE_RULES.iter().find(|rule| rule.component == component)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_input_state_is_host_owned() {
        let rule = state_rule("TextInput").unwrap();

        assert_eq!(rule.ownership, StateOwnership::HostOwned);
    }

    #[test]
    fn scroll_surface_keeps_only_window_keyed_visual_state() {
        let rule = state_rule("ScrollSurface").unwrap();

        assert_eq!(rule.ownership, StateOwnership::WindowKeyed);
    }
}
