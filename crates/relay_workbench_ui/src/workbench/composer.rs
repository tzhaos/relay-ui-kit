use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled,
    Window, div, prelude::FluentBuilder, px,
};

use relay_ui_primitives::theme::{ActiveTheme, radius};

// --- Layout constants ---
/// Min composer height when docked (non-floating).
const MIN_HEIGHT_DOCKED: f32 = 116.0;
/// Min composer height when floating (elevated).
const MIN_HEIGHT_FLOATING: f32 = 124.0;
/// Input area min height when docked.
const INPUT_MIN_HEIGHT_DOCKED: f32 = 72.0;
/// Input area min height when floating.
const INPUT_MIN_HEIGHT_FLOATING: f32 = 78.0;
/// Bottom action bar height when docked.
const BAR_HEIGHT_DOCKED: f32 = 40.0;
/// Bottom action bar height when floating.
const BAR_HEIGHT_FLOATING: f32 = 44.0;
/// Bottom action bar horizontal padding when docked.
const BAR_PX_DOCKED: f32 = 8.0;
/// Bottom action bar horizontal padding when floating.
const BAR_PX_FLOATING: f32 = 16.0;

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
            .min_h(px(if floating {
                MIN_HEIGHT_FLOATING
            } else {
                MIN_HEIGHT_DOCKED
            }))
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
                    .min_h(px(if floating {
                        INPUT_MIN_HEIGHT_FLOATING
                    } else {
                        INPUT_MIN_HEIGHT_DOCKED
                    }))
                    .p_3()
                    .flex()
                    .items_start()
                    .child(self.input),
            )
            .child(
                div()
                    .h(px(if floating {
                        BAR_HEIGHT_FLOATING
                    } else {
                        BAR_HEIGHT_DOCKED
                    }))
                    .px(if floating {
                        px(BAR_PX_FLOATING)
                    } else {
                        px(BAR_PX_DOCKED)
                    })
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
