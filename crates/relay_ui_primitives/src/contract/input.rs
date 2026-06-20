/// The kind of value an input control holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputValueKind {
    /// Free-form text.
    Text,
    /// A numeric value.
    Number,
    /// A value chosen from a discrete set of options.
    Selection,
    /// A boolean on/off value.
    Toggle,
}

/// An action performed on or by an input control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputActionKind {
    /// The input value changed, carrying the kind of value that was mutated.
    Changed(InputValueKind),
    /// The cursor or caret position moved without changing the value.
    CursorMoved,
    /// The input was submitted (e.g. Enter key).
    Submit,
    /// The input was cancelled (e.g. Escape key).
    Cancel,
    /// An explicit validation was requested.
    Validate,
    /// The action was swallowed and should be ignored.
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

/// The validation state of an input's current value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationState {
    /// The value has not yet been validated (pristine state).
    NotValidated,
    /// The value passed validation.
    Valid,
    /// The value failed validation.
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
