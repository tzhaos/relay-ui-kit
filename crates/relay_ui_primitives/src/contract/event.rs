#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Click,
    Select,
    Change,
    Dismiss,
    Submit,
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
