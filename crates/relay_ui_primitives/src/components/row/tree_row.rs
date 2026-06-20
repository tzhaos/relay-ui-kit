use gpui::{
    App, ClickEvent, ElementId, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::ClickHandler,
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
            on_click: None,
        }
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

    crate::callback_builder!(on_click, on_click, ClickEvent);
}

impl RenderOnce for TreeRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let fg = if self.selected {
            theme.text
        } else {
            theme.text_secondary
        };
        let chevron = if self.expandable {
            Some(if self.expanded {
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
            .selected(self.selected)
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

        if let Some(handler) = self.on_click {
            row = row.on_click_handler(handler);
        }

        row
    }
}
