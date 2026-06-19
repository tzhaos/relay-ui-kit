use super::Layer;

/// Who owns the mutable state for a component.
///
/// - [`HostOwned`]: the parent view owns the state (value flows through the GPUI
///   entity tree). This is the dominant pattern for interactive components.
/// - [`WindowKeyed`]: state is purely visual and keyed to the window (e.g.,
///   scroll position). It persists across renders of the same surface but is
///   not considered semantic state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateOwnership {
    HostOwned,
    WindowKeyed,
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
