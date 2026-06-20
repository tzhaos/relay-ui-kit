use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled,
    Window, div, px,
};

use crate::{theme::{ActiveTheme, radius, space, BORDER_WIDTH}};

/// A compact horizontal group for toolbar icon controls.
#[derive(IntoElement)]
pub struct ToolbarGroup {
    id: ElementId,
    children: Vec<AnyElement>,
}

impl ToolbarGroup {
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

impl RenderOnce for ToolbarGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        div()
            .id(self.id)
            .h(px(30.0))
            .px(px(space::XXS))
            .flex()
            .items_center()
            .gap(px(BORDER_WIDTH))
            .rounded(px(radius::MD))
            .border_1()
            .border_color(theme.border)
            .bg(theme.panel)
            .children(self.children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toolbar_group_starts_empty() {
        let group = ToolbarGroup::new("toolbar");

        assert!(group.is_empty());
    }
}
