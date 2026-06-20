/// Standardised event names used in component callbacks.
///
/// Each variant maps to a canonical `on_<name>` handler function name
/// (e.g. [`Click`](EventKind::Click) → `on_click`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// A pointer click or equivalent activation.
    Click,
    /// An item or value was selected from a set of options.
    Select,
    /// The value of an input control changed.
    Change,
    /// An overlay (menu, popover, dialog) was dismissed.
    Dismiss,
    /// A form or action was submitted.
    Submit,
    /// A form or action was cancelled.
    Cancel,
}

impl EventKind {
    pub fn handler_name(self) -> &'static str {
        match self {
            EventKind::Click => "on_click",
            EventKind::Select => "on_select",
            EventKind::Change => "on_change",
            EventKind::Dismiss => "on_dismiss",
            EventKind::Submit => "on_submit",
            EventKind::Cancel => "on_cancel",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_handler_names_are_stable() {
        assert_eq!(EventKind::Dismiss.handler_name(), "on_dismiss");
    }
}
