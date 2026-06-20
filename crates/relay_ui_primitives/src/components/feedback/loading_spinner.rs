use gpui::{
    Animation, AnimationExt, App, ElementId, IntoElement, ParentElement, RenderOnce, Styled,
    Transformation, Window, div, percentage, prelude::FluentBuilder, px, svg,
};

use crate::{contract::MotionDuration, icon::IconName, theme::ActiveTheme, tone::Tone};

/// An inline spinner for indeterminate work.
#[derive(IntoElement)]
pub struct LoadingSpinner {
    id: ElementId,
    label: Option<String>,
    tone: Tone,
}

impl LoadingSpinner {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            tone: Tone::Muted,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
}

impl RenderOnce for LoadingSpinner {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let fg = self.tone.fg(&theme);
        let id = self.id.clone();
        let glyph = svg()
            .path(IconName::RefreshCw.path())
            .size(px(14.0))
            .text_color(fg)
            .with_animation(
                (id, "loading-spinner"),
                Animation::new(MotionDuration::Slow.duration()).repeat(),
                |this, delta| this.with_transformation(Transformation::rotate(percentage(delta))),
            );

        div()
            .h(px(24.0))
            .flex()
            .items_center()
            .gap_1()
            .text_color(theme.text_secondary)
            .child(
                div()
                    .size(px(16.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(glyph),
            )
            .when_some(self.label, |this, label| {
                this.child(div().text_xs().child(label))
            })
    }
}
