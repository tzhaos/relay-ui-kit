use gpui::{
    App, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled,
    Window, div, px,
};

use crate::theme::{ActiveTheme, radius};

/// A fixed-size color chip.
#[derive(IntoElement)]
pub struct ColorSwatch {
    id: ElementId,
    color: Hsla,
}

impl ColorSwatch {
    pub fn new(id: impl Into<ElementId>, color: Hsla) -> Self {
        Self {
            id: id.into(),
            color,
        }
    }
}

impl RenderOnce for ColorSwatch {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .id(self.id)
            .size(px(18.0))
            .rounded(px(radius::SM))
            .bg(self.color)
            .border_1()
            .border_color(cx.theme().border_strong)
    }
}

/// A compact color value field with a leading swatch.
#[derive(IntoElement)]
pub struct ColorField {
    id: ElementId,
    color: Hsla,
    value: String,
}

impl ColorField {
    pub fn new(id: impl Into<ElementId>, color: Hsla, value: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            color,
            value: value.into(),
        }
    }
}

impl RenderOnce for ColorField {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let id = self.id;
        div()
            .id(id.clone())
            .h(px(30.0))
            .min_w(px(148.0))
            .px_2()
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(radius::MD))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border)
            .child(ColorSwatch::new((id, "swatch"), self.color))
            .child(div().text_sm().text_color(theme.text).child(self.value))
    }
}
