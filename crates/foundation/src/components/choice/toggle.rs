use gpui::{
    App, ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Role,
    StatefulInteractiveElement, Styled, Toggled, Window, div, prelude::FluentBuilder, px,
};

use crate::{
    interaction::ClickHandler,
    theme::{ActiveTheme, space},
};

/// A sliding on/off switch. The host owns `on` and flips it in `on_click`.
#[derive(IntoElement)]
pub struct Toggle {
    id: ElementId,
    on: bool,
    label: Option<String>,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl Toggle {
    pub fn new(id: impl Into<ElementId>, on: bool) -> Self {
        Self {
            id: id.into(),
            on,
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

    crate::callback_builder!(on_click, on_click, ClickEvent);
}

impl RenderOnce for Toggle {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let track_bg = if self.on {
            theme.accent
        } else {
            theme.border_strong
        };
        let disabled = self.disabled;
        let handler = self.on_click.filter(|_| !disabled);

        let track = div()
            .w(px(32.0))
            .h(px(18.0))
            .flex_shrink_0()
            .rounded(px(9.0))
            .bg(track_bg)
            .p(px(space::XXS))
            .flex()
            .items_center()
            .when(self.on, |this| this.justify_end())
            .when(!self.on, |this| this.justify_start())
            .child(div().size(px(14.0)).rounded(px(7.0)).bg(theme.panel));

        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap_2()
            .role(Role::Switch)
            .aria_toggled(Toggled::from(self.on))
            .when(disabled, |this| this.opacity(0.5))
            .when(!disabled, |this| this.cursor_pointer())
            .child(track)
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
