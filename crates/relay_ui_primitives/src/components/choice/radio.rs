use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{interaction::ClickHandler, theme::ActiveTheme};

/// A labelled radio option. The host renders a group and tracks the selection.
#[derive(IntoElement)]
pub struct Radio {
    id: ElementId,
    selected: bool,
    label: String,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl Radio {
    pub fn new(id: impl Into<ElementId>, selected: bool, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            selected,
            label: label.into(),
            disabled: false,
            on_click: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

impl RenderOnce for Radio {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let border = if self.selected {
            theme.accent
        } else {
            theme.border_strong
        };
        let disabled = self.disabled;
        let handler = self.on_click.filter(|_| !disabled);

        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap_2()
            .when(disabled, |this| this.opacity(0.5))
            .when(!disabled, |this| this.cursor_pointer())
            .child(
                div()
                    .size(px(16.0))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(border)
                    .bg(theme.panel)
                    .when(self.selected, |this| {
                        this.child(div().size(px(8.0)).rounded(px(4.0)).bg(theme.accent))
                    }),
            )
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text)
                    .child(self.label),
            )
            .when_some(handler, |this, handler| {
                this.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}
