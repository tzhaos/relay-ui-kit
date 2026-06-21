use gpui::{DefiniteLength, MouseButton};

use crate::component_prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListItemSpacing {
    Compact,
    Dense,
    Relaxed,
}

impl ListItemSpacing {
    fn height(self) -> f32 {
        match self {
            Self::Compact => space::ROW_SM,
            Self::Dense => space::ROW_MD,
            Self::Relaxed => space::TASK_ROW,
        }
    }
}

#[derive(IntoElement)]
pub struct ListItem {
    id: ElementId,
    selected: bool,
    disabled: bool,
    selectable: bool,
    spacing: ListItemSpacing,
    height: Option<DefiniteLength>,
    indent_depth: usize,
    indent_step: f32,
    start_slot: Option<AnyElement>,
    end_slot: Option<AnyElement>,
    on_click: Option<ClickHandler>,
    children: Vec<AnyElement>,
}

impl ListItem {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            selected: false,
            disabled: false,
            selectable: true,
            spacing: ListItemSpacing::Dense,
            height: None,
            indent_depth: 0,
            indent_step: 14.0,
            start_slot: None,
            end_slot: None,
            on_click: None,
            children: Vec::new(),
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    pub fn spacing(mut self, spacing: ListItemSpacing) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn height(mut self, height: impl Into<DefiniteLength>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn indent(mut self, depth: usize, step: f32) -> Self {
        self.indent_depth = depth;
        self.indent_step = step;
        self
    }

    pub fn start_slot(mut self, slot: impl IntoElement) -> Self {
        self.start_slot = Some(slot.into_any_element());
        self
    }

    pub fn end_slot(mut self, slot: impl IntoElement) -> Self {
        self.end_slot = Some(slot.into_any_element());
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

impl ParentElement for ListItem {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ListItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let height = self
            .height
            .unwrap_or_else(|| px(self.spacing.height()).into());
        let disabled = self.disabled;
        let selected = self.selected;
        let clickable = self.on_click.is_some() && !disabled;
        let selectable = self.selectable && !disabled;

        div()
            .id(self.id)
            .w_full()
            .h(height)
            .min_w_0()
            .pl(px(space::SM + self.indent_depth as f32 * self.indent_step))
            .pr_2()
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(radius::MD))
            .border_1()
            .border_color(if selected {
                theme.accent_border
            } else {
                gpui::transparent_black()
            })
            .text_color(if disabled {
                theme.text_muted.opacity(0.55)
            } else if selected {
                theme.text
            } else {
                theme.text_secondary
            })
            .when(selected, |this| this.bg(theme.selection))
            .when(selectable && !selected, |this| {
                this.hover(move |style| style.bg(theme.hover))
            })
            .when(clickable, |this| {
                this.cursor_pointer()
                    .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                        window.prevent_default();
                    })
            })
            .children(self.start_slot)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .gap_2()
                    .children(self.children),
            )
            .children(self.end_slot)
            .when_some(self.on_click.filter(|_| clickable), |this, handler| {
                this.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_item_defaults_to_selectable_dense_row() {
        let item = ListItem::new("list-item");

        assert_eq!(item.spacing, ListItemSpacing::Dense);
    }
}
