use crate::icon::IconName;

/// One selectable item row inside an [`super::ItemPicker`].
pub struct PickerOption<K> {
    pub(super) key: K,
    pub(super) label: String,
    pub(super) detail: Option<String>,
    pub(super) icon: Option<IconName>,
}

impl<K> PickerOption<K> {
    /// Create an item option with a stable key and visible label.
    pub fn new(key: K, label: impl Into<String>) -> Self {
        Self {
            key,
            label: label.into(),
            detail: None,
            icon: None,
        }
    }

    /// Add supporting detail text below the main option label.
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Add a leading icon for trigger and panel presentation.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// One secondary action row shown under an [`super::ItemPicker`] item list.
pub struct PickerAction {
    pub(super) key: String,
    pub(super) label: String,
    pub(super) icon: IconName,
}

impl PickerAction {
    /// Create an action with a stable key, visible label, and leading icon.
    pub fn new(key: impl Into<String>, label: impl Into<String>, icon: IconName) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            icon,
        }
    }
}

/// Standard contextual actions used by picker-adjacent action menus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerActionKind {
    Checkout,
    NewWorktree,
    Rename,
    Delete,
}

impl PickerActionKind {
    /// User-facing action label for menus and gallery examples.
    pub fn label(self) -> &'static str {
        match self {
            PickerActionKind::Checkout => "Checkout item",
            PickerActionKind::NewWorktree => "New worktree from item",
            PickerActionKind::Rename => "Rename item",
            PickerActionKind::Delete => "Delete item",
        }
    }

    /// Leading icon that matches the action intent.
    pub fn icon(self) -> IconName {
        match self {
            PickerActionKind::Checkout => IconName::GitBranch,
            PickerActionKind::NewWorktree => IconName::FolderPlus,
            PickerActionKind::Rename => IconName::Settings,
            PickerActionKind::Delete => IconName::Archive,
        }
    }

    /// Whether the action should be rendered in a danger tone.
    pub fn is_dangerous(self) -> bool {
        matches!(self, PickerActionKind::Delete)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_picker_action_is_dangerous() {
        assert!(PickerActionKind::Delete.is_dangerous());
    }

    #[test]
    fn picker_option_icon_builder_stores_icon() {
        let option = PickerOption::new("main", "main").icon(IconName::GitBranch);

        assert_eq!(option.icon, Some(IconName::GitBranch));
    }
}
