use gpui::{AnyElement, ClickEvent, IntoElement};

use relay_foundation::{icon::IconName, interaction::ClickHandler};

/// One row in a [`super::Menu`].
pub struct MenuItem {
    pub(super) label: String,
    pub(super) detail: Option<String>,
    pub(super) icon: Option<IconName>,
    pub(super) trailing: Option<AnyElement>,
    pub(super) checked: bool,
    pub(super) danger: bool,
    pub(super) disabled: bool,
    pub(super) separator: bool,
    pub(super) header: bool,
    pub(super) submenu: bool,
    pub(super) submenu_items: Vec<MenuItem>,
    pub(super) submenu_open: bool,
    pub(super) on_click: Option<ClickHandler>,
}

impl MenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            detail: None,
            icon: None,
            trailing: None,
            checked: false,
            danger: false,
            disabled: false,
            separator: false,
            header: false,
            submenu: false,
            submenu_items: Vec::new(),
            submenu_open: false,
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
            checked: false,
            danger: false,
            disabled: false,
            separator: true,
            header: false,
            submenu: false,
            submenu_items: Vec::new(),
            submenu_open: false,
            on_click: None,
        }
    }

    pub fn header(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            detail: None,
            icon: None,
            trailing: None,
            checked: false,
            danger: false,
            disabled: true,
            separator: false,
            header: true,
            submenu: false,
            submenu_items: Vec::new(),
            submenu_open: false,
            on_click: None,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Render in the danger tone.
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn submenu(mut self) -> Self {
        self.submenu = true;
        self
    }

    pub fn submenu_items(mut self, items: Vec<MenuItem>) -> Self {
        self.submenu = true;
        self.submenu_items = items;
        self
    }

    pub fn submenu_open(mut self, open: bool) -> Self {
        self.submenu_open = open;
        self
    }

    relay_foundation::callback_builder!(on_click, on_click, ClickEvent);
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

        assert!(!item.submenu_open);
    }

    #[test]
    fn submenu_open_sets_explicit_submenu_state() {
        let item = MenuItem::new("Open With")
            .submenu_items(vec![MenuItem::new("Shell")])
            .submenu_open(true);

        assert!(item.submenu_open);
    }

    #[test]
    fn menu_header_is_not_interactive() {
        let item = MenuItem::header("Actions");

        assert!(item.header);
        assert!(item.disabled);
    }
}
