#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputValueKind {
    Text,
    Number,
    Selection,
    Toggle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputActionKind {
    Changed(InputValueKind),
    CursorMoved,
    Submit,
    Cancel,
    Validate,
    Ignored,
}

impl InputActionKind {
    pub fn changes_value(self) -> bool {
        matches!(self, Self::Changed(_))
    }

    pub fn should_notify(self) -> bool {
        !matches!(self, Self::Ignored)
    }

    pub fn should_validate(self) -> bool {
        matches!(self, Self::Changed(_) | Self::Submit | Self::Validate)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationState {
    NotValidated,
    Valid,
    Invalid,
}

impl ValidationState {
    pub fn should_show_error(self) -> bool {
        matches!(self, Self::Invalid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changed_input_actions_require_validation() {
        assert!(InputActionKind::Changed(InputValueKind::Text).should_validate());
    }

    #[test]
    fn cursor_motion_notifies_without_changing_value() {
        let action = InputActionKind::CursorMoved;

        assert!(action.should_notify());
        assert!(!action.changes_value());
    }
}
