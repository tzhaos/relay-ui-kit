use crate::icon::IconName;

/// A selectable Git branch row in [`super::BranchSelector`].
pub struct PickerOption {
    pub(super) key: &'static str,
    pub(super) label: String,
    pub(super) detail: Option<String>,
}

impl PickerOption {
    pub fn new(key: &'static str, label: impl Into<String>) -> Self {
        Self {
            key,
            label: label.into(),
            detail: None,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// A non-destructive helper action shown at the bottom of a branch picker.
pub struct PickerAction {
    pub(super) key: &'static str,
    pub(super) label: String,
    pub(super) icon: IconName,
}

impl PickerAction {
    pub fn new(key: &'static str, label: impl Into<String>, icon: IconName) -> Self {
        Self {
            key,
            label: label.into(),
            icon,
        }
    }
}

/// Branch operations shown from a separate "more" menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerActionKind {
    Checkout,
    NewWorktree,
    Rename,
    Delete,
}

impl PickerActionKind {
    pub fn label(self) -> &'static str {
        match self {
            PickerActionKind::Checkout => "Checkout item",
            PickerActionKind::NewWorktree => "New worktree from item",
            PickerActionKind::Rename => "Rename item",
            PickerActionKind::Delete => "Delete item",
        }
    }

    pub fn icon(self) -> IconName {
        match self {
            PickerActionKind::Checkout => IconName::GitBranch,
            PickerActionKind::NewWorktree => IconName::FolderPlus,
            PickerActionKind::Rename => IconName::Settings,
            PickerActionKind::Delete => IconName::Archive,
        }
    }

    pub fn is_dangerous(self) -> bool {
        matches!(self, PickerActionKind::Delete)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_branch_action_is_dangerous() {
        assert!(PickerActionKind::Delete.is_dangerous());
    }
}
