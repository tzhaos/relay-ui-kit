use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::ClickHandler,
    theme::{ActiveTheme, radius},
};

/// A horizontal row for search/filter chips above lists or history views.
#[derive(IntoElement)]
pub struct FilterBar {
    id: ElementId,
    children: Vec<AnyElement>,
}

impl FilterBar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
}

impl RenderOnce for FilterBar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap_2()
            .min_w_0()
            .flex_wrap()
            .children(self.children)
    }
}

/// A compact selectable filter trigger, suitable for project/type/model filters.
#[derive(IntoElement)]
pub struct FilterChip {
    id: ElementId,
    label: String,
    icon: Option<IconName>,
    count: Option<usize>,
    selected: bool,
    open: bool,
    dropdown: bool,
    on_click: Option<ClickHandler>,
}

impl FilterChip {
    pub fn new(id: impl Into<ElementId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            count: None,
            selected: false,
            open: false,
            dropdown: false,
            on_click: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn dropdown(mut self, dropdown: bool) -> Self {
        self.dropdown = dropdown;
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

impl RenderOnce for FilterChip {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let active = self.selected || self.open;
        let fg = if active {
            theme.text
        } else {
            theme.text_secondary
        };
        let icon_color = if active {
            theme.accent
        } else {
            theme.text_muted
        };

        div()
            .id(self.id)
            .h(px(30.0))
            .max_w(px(220.0))
            .px_2()
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(radius::MD))
            .border_1()
            .border_color(if active {
                theme.border_strong
            } else {
                theme.border
            })
            .bg(if active { theme.panel_alt } else { theme.panel })
            .text_color(fg)
            .when_some(self.on_click, |this, handler| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(theme.hover).border_color(theme.border_strong))
                    .on_click(move |event, window, cx| {
                        handler(event, window, cx);
                        cx.stop_propagation();
                    })
            })
            .when_some(self.icon, |this, icon| {
                this.child(Icon::new(icon).size(IconSize::Small).color(icon_color))
            })
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.label),
            )
            .when_some(self.count, |this, count| {
                this.child(
                    div()
                        .min_w(px(18.0))
                        .h(px(18.0))
                        .px_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(radius::SM))
                        .bg(theme.inset)
                        .text_color(theme.text_muted)
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(count.to_string()),
                )
            })
            .when(self.dropdown, |this| {
                this.child(
                    Icon::new(IconName::ChevronDown)
                        .size(IconSize::XSmall)
                        .color(theme.text_muted),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_bar_starts_empty() {
        let bar = FilterBar::new("filters");

        assert!(bar.is_empty());
    }

    #[test]
    fn filter_chip_keeps_count() {
        let chip = FilterChip::new("all", "All sessions").count(8);

        assert_eq!(chip.count, Some(8));
    }
}
