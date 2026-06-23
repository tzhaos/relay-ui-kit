use std::hash::Hash;

use gpui::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, RenderOnce, Role, StatefulInteractiveElement, Styled, Toggled,
    Window, div, prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    interaction::{ClickHandler, SelectionSource},
    theme::{ActiveTheme, DISABLED_OPACITY},
};

/// A labelled radio option. The host renders a group and tracks the selection.
#[derive(IntoElement)]
pub struct Radio<T>
where
    T: Clone + Eq + Hash + PartialEq + 'static,
{
    id: ElementId,
    selected: bool,
    label: String,
    disabled: bool,
    source: Option<SelectionSource<T>>,
    value: Option<T>,
    on_click: Option<ClickHandler>,
}

impl<T> Radio<T>
where
    T: Clone + Eq + Hash + PartialEq + 'static,
{
    pub fn new(id: impl Into<ElementId>, selected: bool, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            selected,
            label: label.into(),
            disabled: false,
            source: None,
            value: None,
            on_click: None,
        }
    }

    pub fn bound(
        id: impl Into<ElementId>,
        binding: Binding<T>,
        value: T,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            selected: false,
            label: label.into(),
            disabled: false,
            source: Some(SelectionSource::binding(binding)),
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

impl<T> RenderOnce for Radio<T>
where
    T: Clone + Eq + Hash + PartialEq + 'static,
{
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let source = self.source;
        let value = self.value;
        let selected = source
            .as_ref()
            .zip(value.as_ref())
            .and_then(|(source, value)| source.get(cx).map(|selected| selected == value.clone()))
            .unwrap_or(self.selected);
        let border = if selected {
            theme.accent
        } else {
            theme.border_strong
        };
        let disabled = self.disabled;
        let handler = self.on_click;
        let interactive = !disabled && (source.is_some() || handler.is_some());

        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap_2()
            .role(Role::RadioButton)
            .aria_label(self.label.clone())
            .aria_selected(selected)
            .aria_toggled(Toggled::from(selected))
            .when(interactive, |this| this.tab_index(0))
            .when(disabled, |this| {
                this.opacity(DISABLED_OPACITY)
                    .cursor(gpui::CursorStyle::OperationNotAllowed)
            })
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
                let source_for_click = source.clone();
                let source_for_key = source;
                let value_for_click = value.clone();
                let value_for_key = value;
                let handler_for_click = handler.map(std::rc::Rc::new);
                let handler_for_key = handler_for_click.clone();
                this.on_click(move |event, window, cx| {
                    if let Some((source, value)) =
                        source_for_click.as_ref().zip(value_for_click.as_ref())
                    {
                        source.select(cx, value.clone());
                    }
                    if let Some(handler) = &handler_for_click {
                        handler(event, window, cx);
                    }
                    cx.stop_propagation();
                })
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if event.keystroke.key.as_str() == " "
                        || event.keystroke.key.as_str() == "enter"
                    {
                        if let Some((source, value)) =
                            source_for_key.as_ref().zip(value_for_key.as_ref())
                        {
                            source.select(cx, value.clone());
                        }
                        if let Some(handler) = &handler_for_key {
                            handler(&ClickEvent::default(), window, cx);
                        }
                        cx.stop_propagation();
                    }
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

        assert!(radio.source.is_some());
        assert_eq!(radio.value, Some("dark"));
    }
}
