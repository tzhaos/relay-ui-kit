use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    components::display::CountBadge,
    icon::{Icon, IconName, IconSize},
    interaction::ClickHandler,
    theme::{ActiveTheme, radius, space},
    tone::Tone,
};

/// A disclosure row for collapsible groups.
#[derive(IntoElement)]
pub struct Disclosure {
    id: ElementId,
    label: String,
    open: bool,
    detail: Option<String>,
    count: Option<usize>,
    binding: Option<Binding<bool>>,
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
            binding: None,
            on_toggle: None,
        }
    }

    pub fn bound(
        id: impl Into<ElementId>,
        label: impl Into<String>,
        binding: Binding<bool>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            open: false,
            detail: None,
            count: None,
            binding: Some(binding),
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
        let binding = self.binding;
        let open = binding.as_ref().map_or(self.open, |b| b.get(cx));
        let handler = self.on_toggle;
        let interactive = binding.is_some() || handler.is_some();

        div()
            .id(self.id)
            .h(px(30.0))
            .px(px(space::SM))
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(radius::MD))
            .text_color(theme.text_secondary)
            .when(interactive, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(theme.hover))
                    .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                        window.prevent_default();
                    })
            })
            .child(
                Icon::new(if open {
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
            .when(interactive, |this| {
                this.on_click(move |event, window, cx| {
                    if let Some(binding) = &binding {
                        binding.update(cx, |open| {
                            *open = !*open;
                            true
                        });
                    }
                    if let Some(handler) = &handler {
                        handler(event, window, cx);
                    }
                    cx.stop_propagation();
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use gpui::TestApp;
    use relay::ReactiveAppExt;

    use super::*;

    #[test]
    fn disclosure_stores_open_state_from_host() {
        let disclosure = Disclosure::new("group", "Sessions", true);

        assert!(disclosure.open);
    }

    #[test]
    fn bound_disclosure_stores_binding() {
        let mut app = TestApp::new();
        let disclosure = app.update(|cx| Disclosure::bound("group", "Sessions", cx.binding(false)));

        assert!(disclosure.binding.is_some());
    }
}
