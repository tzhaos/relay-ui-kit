use gpui::{
    App, ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::ClickHandler,
    theme::{ActiveTheme, radius},
};

/// A labelled checkbox. The host owns `checked` and toggles it in `on_click`.
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    checked: bool,
    label: Option<String>,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>, checked: bool) -> Self {
        Self {
            id: id.into(),
            checked,
            label: None,
            disabled: false,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
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

impl RenderOnce for Checkbox {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let (box_bg, box_border) = if self.checked {
            (theme.accent, theme.accent)
        } else {
            (theme.panel, theme.border_strong)
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
                    .rounded(px(radius::SM))
                    .border_1()
                    .border_color(box_border)
                    .bg(box_bg)
                    .when(self.checked, |this| {
                        this.child(
                            Icon::new(IconName::Check)
                                .size(IconSize::XSmall)
                                .color(theme.on_accent),
                        )
                    }),
            )
            .when_some(self.label, |this, label| {
                this.child(div().text_sm().text_color(theme.text).child(label))
            })
            .when_some(handler, |this, handler| {
                this.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                    cx.stop_propagation();
                })
            })
    }
}
