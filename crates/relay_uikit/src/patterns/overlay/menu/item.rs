use gpui::{AnyElement, App, ClickEvent, IntoElement, Window};

use crate::{icon::IconName, interaction::SharedClickHandler};

/// One row in a [`super::Menu`].
pub struct MenuItem {
    pub(super) label: String,
    pub(super) detail: Option<String>,
    pub(super) icon: Option<IconName>,
    pub(super) trailing: Option<AnyElement>,
    pub(super) checkable: bool,
    pub(super) checked: bool,
    pub(super) danger: bool,
    pub(super) disabled: bool,
    pub(super) separator: bool,
    pub(super) header: bool,
    pub(super) submenu: bool,
    pub(super) submenu_items: Vec<MenuItem>,
    pub(super) on_click: Option<SharedClickHandler>,
}

impl MenuItem {
    /// Create an interactive menu row with a visible label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            detail: None,
            icon: None,
            trailing: None,
            checkable: false,
            checked: false,
            danger: false,
            disabled: false,
            separator: false,
            header: false,
            submenu: false,
            submenu_items: Vec::new(),
            on_click: None,
        }
    }

    /// A 1px divider row between groups of items.
    pub fn separator() -> Self {
        Self {
            label: String::new(),
            detail: None,
            icon: None,
            trailing: None,
            checkable: false,
            checked: false,
            danger: false,
            disabled: false,
            separator: true,
            header: false,
            submenu: false,
            submenu_items: Vec::new(),
            on_click: None,
        }
    }

    /// Create a non-interactive group heading row.
    pub fn header(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            detail: None,
            icon: None,
            trailing: None,
            checkable: false,
            checked: false,
            danger: false,
            disabled: true,
            separator: false,
            header: true,
            submenu: false,
            submenu_items: Vec::new(),
            on_click: None,
        }
    }

    /// Add supporting detail text below the main label.
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Add a leading icon to the menu row.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Render trailing affordances such as shortcuts or badges.
    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }

    /// Mark the row as currently selected or checked.
    ///
    /// This also marks the row as a checkbox-style menu item for assistive
    /// technologies, even when `checked` is `false`.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checkable = true;
        self.checked = checked;
        self
    }

    /// Render in the danger tone.
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    /// Disable activation while keeping the row visible.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Reserve submenu affordance even before submenu items are attached.
    pub fn submenu(mut self) -> Self {
        self.submenu = true;
        self
    }

    /// Attach a nested submenu that opens from this row.
    pub fn submenu_items(mut self, items: Vec<MenuItem>) -> Self {
        self.submenu = true;
        self.submenu_items = items;
        self
    }

    /// Observe row activation from both pointer and keyboard menu interaction.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(std::rc::Rc::new(handler));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submenu_items_mark_item_as_submenu() {
        let item = MenuItem::new("Open With").submenu_items(vec![MenuItem::new("Shell")]);

        assert!(item.submenu);
    }

    #[test]
    fn submenu_items_do_not_open_submenu_by_default() {
        let item = MenuItem::new("Open With").submenu_items(vec![MenuItem::new("Shell")]);

        assert_eq!(item.submenu_items.len(), 1);
    }

    #[test]
    fn menu_header_is_not_interactive() {
        let item = MenuItem::header("Actions");

        assert!(item.header);
        assert!(item.disabled);
    }

    #[test]
    fn checked_builder_marks_item_as_checkable_even_when_false() {
        let item = MenuItem::new("Auto save").checked(false);

        assert!(item.checkable);
        assert!(!item.checked);
    }
}
