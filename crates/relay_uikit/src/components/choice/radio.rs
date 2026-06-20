use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, Role, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    interaction::ClickHandler,
    theme::{ActiveTheme, DISABLED_OPACITY},
};

/// A labelled radio option. The host renders a group and tracks the selection.
#[derive(IntoElement)]
pub struct Radio {
    id: ElementId,
    selected: bool,
    label: String,
    disabled: bool,
    binding: Option<Binding<&'static str>>,
    value: Option<&'static str>,
    on_click: Option<ClickHandler>,
}

impl Radio {
    pub fn new(id: impl Into<ElementId>, selected: bool, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            selected,
            label: label.into(),
            disabled: false,
            binding: None,
            value: None,
            on_click: None,
        }
    }

    pub fn bound(
        id: impl Into<ElementId>,
        binding: Binding<&'static str>,
        value: &'static str,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            selected: false,
            label: label.into(),
            disabled: false,
            binding: Some(binding),
            value: Some(value),
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
        let binding = self.binding;
        let value = self.value;
        let selected = binding
            .as_ref()
            .zip(value)
            .map_or(self.selected, |(binding, value)| binding.get(cx) == value);
        let border = if selected {
            theme.accent
        } else {
            theme.border_strong
        };
        let disabled = self.disabled;
        let handler = self.on_click;
        let interactive = !disabled && (binding.is_some() || handler.is_some());

        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap_2()
            .role(Role::RadioButton)
            .when(disabled, |this| this.opacity(DISABLED_OPACITY))
            .when(interactive, |this| {
                this.cursor_pointer()
                    .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                        window.prevent_default();
                    })
            })
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
                    .when(selected, |this| {
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
            .when(interactive, |this| {
                this.on_click(move |event, window, cx| {
                    if let Some((binding, value)) = binding.as_ref().zip(value) {
                        binding.set(cx, value);
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
    fn bound_radio_stores_binding_and_value() {
        let mut app = TestApp::new();
        let radio = app.update(|cx| Radio::bound("radio", cx.binding("light"), "dark", "Dark"));

        assert!(radio.binding.is_some());
        assert_eq!(radio.value, Some("dark"));
    }
}
