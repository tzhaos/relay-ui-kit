use gpui::{
    App, ClickEvent, ElementId, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::Binding;

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
    selected_binding: Option<Binding<bool>>,
    expanded_binding: Option<Binding<bool>>,
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
            selected_binding: None,
            expanded_binding: None,
            on_click: None,
        }
    }

    pub fn bound(
        id: impl Into<ElementId>,
        icon: IconName,
        label: impl Into<String>,
        selected: Binding<bool>,
    ) -> Self {
        Self {
            id: id.into(),
            icon,
            label: label.into(),
            depth: 0,
            expandable: false,
            expanded: false,
            selected: false,
            selected_binding: Some(selected),
            expanded_binding: None,
            on_click: None,
        }
    }

    pub fn expanded_bound(mut self, binding: Binding<bool>) -> Self {
        self.expanded_binding = Some(binding);
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
        let selected_binding = self.selected_binding;
        let expanded_binding = self.expanded_binding;
        let selected = selected_binding.as_ref().map_or(self.selected, |b| b.get(cx));
        let expanded = expanded_binding.as_ref().map_or(self.expanded, |b| b.get(cx));
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

        let has_click = selected_binding.is_some() || expanded_binding.is_some() || self.on_click.is_some();
        if has_click {
            let handler = self.on_click;
            row = row.on_click(move |event, window, cx| {
                if let Some(binding) = &selected_binding {
                    binding.update(cx, |selected| {
                        *selected = !*selected;
                        true
                    });
                }
                if let Some(binding) = &expanded_binding {
                    binding.update(cx, |expanded| {
                        *expanded = !*expanded;
                        true
                    });
                }
                if let Some(handler) = &handler {
                    handler(event, window, cx);
                }
            });
        }

        row
    }
}
