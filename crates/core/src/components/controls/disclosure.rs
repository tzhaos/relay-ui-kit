use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};

use crate::{
    components::display::CountBadge,
    icon::{Icon, IconName, IconSize},
    interaction::ClickHandler,
    theme::{ActiveTheme, radius, space},
    tone::Tone,
};

/// A host-owned disclosure row for collapsible groups.
#[derive(IntoElement)]
pub struct Disclosure {
    id: ElementId,
    label: String,
    open: bool,
    detail: Option<String>,
    count: Option<usize>,
    on_toggle: Option<ClickHandler>,
}

impl Disclosure {
    pub fn new(id: impl Into<ElementId>, label: impl Into<String>, open: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            open,
            detail: None,
            count: None,
            on_toggle: None,
        }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    pub fn on_toggle(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Disclosure {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let handler = self.on_toggle;

        div()
            .id(self.id)
            .h(px(30.0))
            .px(px(space::SM))
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(radius::MD))
            .text_color(theme.text_secondary)
            .when(handler.is_some(), |this| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(theme.hover))
                    .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                        window.prevent_default();
                    })
            })
            .child(
                Icon::new(if self.open {
                    IconName::ChevronDown
                } else {
                    IconName::ChevronRight
                })
                .size(IconSize::XSmall)
                .color(theme.text_muted),
            )
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .truncate()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.label),
            )
            .when_some(self.detail, |this, detail| {
                this.child(
                    div()
                        .max_w(px(160.0))
                        .truncate()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child(detail),
                )
            })
            .when_some(self.count, |this, count| {
                this.child(CountBadge::new(count).tone(Tone::Secondary))
            })
            .when_some(handler, |this, handler| {
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
    fn disclosure_stores_open_state_from_host() {
        let disclosure = Disclosure::new("group", "Sessions", true);

        assert!(disclosure.open);
    }
}
