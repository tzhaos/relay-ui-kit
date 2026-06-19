use relay_ui_primitives::icon::IconName;

/// A selectable Git branch row in [`super::BranchSelector`].
pub struct BranchOption {
    pub(super) key: &'static str,
    pub(super) label: String,
    pub(super) detail: Option<String>,
}

impl BranchOption {
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
pub struct BranchPickerAction {
    pub(super) key: &'static str,
    pub(super) label: String,
    pub(super) icon: IconName,
}

impl BranchPickerAction {
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
pub enum BranchActionKind {
    Checkout,
    NewWorktree,
    Rename,
    Delete,
}

impl BranchActionKind {
    pub fn label(self) -> &'static str {
        match self {
            BranchActionKind::Checkout => "Checkout branch",
            BranchActionKind::NewWorktree => "New worktree from branch",
            BranchActionKind::Rename => "Rename branch",
            BranchActionKind::Delete => "Delete branch",
        }
    }

    pub fn icon(self) -> IconName {
        match self {
            BranchActionKind::Checkout => IconName::GitBranch,
            BranchActionKind::NewWorktree => IconName::FolderPlus,
            BranchActionKind::Rename => IconName::Settings,
            BranchActionKind::Delete => IconName::Archive,
        }
    }

    pub fn is_dangerous(self) -> bool {
        matches!(self, BranchActionKind::Delete)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_branch_action_is_dangerous() {
        assert!(BranchActionKind::Delete.is_dangerous());
    }
}
