use gpui::{
    App, ClickEvent, ElementId, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{ClickHandler, OpenState, SelectionBinding},
    list::ListItem,
    theme::{ActiveTheme, space},
};

/// A file/worktree tree node with indentation and optional disclosure chevron.
#[derive(IntoElement)]
pub struct TreeRow {
    id: ElementId,
    icon: IconName,
    label: String,
    depth: usize,
    expandable: bool,
    expanded: bool,
    selected: bool,
    selection: Option<SelectionBinding>,
    open_state: Option<OpenState>,
    on_click: Option<ClickHandler>,
}

impl TreeRow {
    pub fn new(id: impl Into<ElementId>, icon: IconName, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            icon,
            label: label.into(),
            depth: 0,
            expandable: false,
            expanded: false,
            selected: false,
            selection: None,
            open_state: None,
            on_click: None,
        }
    }

    /// Drive the selected state from a Relay-aware keyed selection controller.
    pub fn selection_binding(mut self, selection: SelectionBinding) -> Self {
        self.selection = Some(selection);
        self
    }

    /// Drive the expanded state from a shared open-state controller.
    pub fn open_state(mut self, open_state: OpenState) -> Self {
        self.open_state = Some(open_state);
        self
    }

    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    pub fn expandable(mut self, expanded: bool) -> Self {
        self.expandable = true;
        self.expanded = expanded;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for TreeRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let selection = self.selection;
        let open_state = self.open_state;
        let selected = selection
            .as_ref()
            .map_or(self.selected, |selection| selection.is_selected(cx));
        let expanded = open_state
            .as_ref()
            .map_or(self.expanded, |open_state| open_state.get(cx));
        let fg = if selected {
            theme.text
        } else {
            theme.text_secondary
        };
        let chevron = if self.expandable {
            Some(if expanded {
                IconName::ChevronDown
            } else {
                IconName::ChevronRight
            })
        } else {
            None
        };

        let start_slot = div()
            .flex()
            .items_center()
            .gap_1()
            .child(
                div()
                    .w(px(14.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .when_some(chevron, |this, chevron| {
                        this.child(
                            Icon::new(chevron)
                                .size(IconSize::XSmall)
                                .color(theme.text_muted),
                        )
                    }),
            )
            .child(
                Icon::new(self.icon)
                    .size(IconSize::Small)
                    .color(theme.text_muted),
            );

        let mut row = ListItem::new(self.id)
            .height(px(space::ROW_SM))
            .indent(self.depth, 14.0)
            .selected(selected)
            .start_slot(start_slot)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_sm()
                    .text_color(fg)
                    .child(self.label),
            );

        let has_click = selection.is_some() || open_state.is_some() || self.on_click.is_some();
        if has_click {
            let handler = self.on_click;
            row = row.on_click(move |event, window, cx| {
                if let Some(selection) = &selection {
                    selection.select(cx);
                }
                if let Some(open_state) = &open_state {
                    open_state.toggle(cx);
                }
                if let Some(handler) = &handler {
                    handler(event, window, cx);
                }
            });
        }

        row
    }
}

#[cfg(test)]
mod tests {
    use gpui::TestApp;
    use relay::ReactiveAppExt;

    use super::*;

    #[test]
    fn tree_row_selection_binding_builder_stores_controller() {
        let mut app = TestApp::new();
        let row = app.update(|cx| {
            TreeRow::new("tree", IconName::Folder, "src")
                .selection_binding(SelectionBinding::binding(cx.binding(true), true))
        });

        assert!(row.selection.is_some());
    }

    #[test]
    fn tree_row_open_state_builder_stores_controller() {
        let mut app = TestApp::new();
        let row = app.update(|cx| {
            TreeRow::new("tree", IconName::Folder, "src")
                .open_state(OpenState::binding(cx.binding(false)))
        });

        assert!(row.open_state.is_some());
    }
}
