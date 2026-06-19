use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled,
    Window, div, prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, radius};

/// A prompt/composer shell for terminal and agent launch flows.
#[derive(IntoElement)]
pub struct Composer {
    id: ElementId,
    input: AnyElement,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    footer: Option<AnyElement>,
    floating: bool,
}

impl Composer {
    pub fn new(id: impl Into<ElementId>, input: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            input: input.into_any_element(),
            leading: None,
            trailing: None,
            footer: None,
            floating: false,
        }
    }

    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.leading = Some(leading.into_any_element());
        self
    }

    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }

    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    pub fn floating(mut self, floating: bool) -> Self {
        self.floating = floating;
        self
    }
}

impl RenderOnce for Composer {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let floating = self.floating;
        div()
            .id(self.id)
            .min_h(px(if floating { 124.0 } else { 116.0 }))
            .rounded(px(radius::LG))
            .bg(theme.panel)
            .border_1()
            .border_color(if floating {
                theme.border
            } else {
                theme.border_strong
            })
            .when(floating, |this| this.shadow_lg())
            .when(!floating, |this| this.shadow_sm())
            .when(!floating, |this| this.overflow_hidden())
            .flex()
            .flex_col()
            .child(
                div()
                    .min_h(px(if floating { 78.0 } else { 72.0 }))
                    .p_3()
                    .flex()
                    .items_start()
                    .child(self.input),
            )
            .child(
                div()
                    .h(px(if floating { 44.0 } else { 40.0 }))
                    .px(if floating { px(16.0) } else { px(8.0) })
                    .flex()
                    .items_center()
                    .justify_between()
                    .when(!floating, |this| {
                        this.border_t_1()
                            .border_color(theme.border)
                            .bg(theme.chrome)
                    })
                    .when(floating, |this| this.bg(theme.panel))
                    .child(
                        div()
                            .min_w_0()
                            .flex()
                            .items_center()
                            .gap_2()
                            .when_some(self.leading, |this, leading| this.child(leading)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .when_some(self.trailing, |this, trailing| this.child(trailing)),
                    ),
            )
            .when_some(self.footer, |this, footer| {
                this.child(
                    div()
                        .px_3()
                        .py_2()
                        .border_t_1()
                        .border_color(theme.border)
                        .child(footer),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composer_defaults_to_non_floating() {
        let composer = Composer::new("composer", div());

        assert!(!composer.floating);
    }
}
