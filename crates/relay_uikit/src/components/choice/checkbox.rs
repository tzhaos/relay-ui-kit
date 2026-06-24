use gpui::{
    App, ClickEvent, ElementId, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, RenderOnce, Role, StatefulInteractiveElement, Styled, Toggled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::ClickHandler,
    theme::{ActiveTheme, DISABLED_OPACITY, radius},
};

/// A labelled checkbox. The host owns `checked` and toggles it in `on_click`.
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    checked: bool,
    label: Option<String>,
    aria_label: Option<String>,
    disabled: bool,
    binding: Option<Binding<bool>>,
    on_click: Option<ClickHandler>,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>, checked: bool) -> Self {
        Self {
            id: id.into(),
            checked,
            label: None,
            aria_label: None,
            disabled: false,
            binding: None,
            on_click: None,
        }
    }

    pub fn bound(id: impl Into<ElementId>, binding: Binding<bool>) -> Self {
        Self {
            id: id.into(),
            checked: false,
            label: None,
            aria_label: None,
            disabled: false,
            binding: Some(binding),
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.aria_label = Some(label.into());
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
        let label = self.label;
        let aria_label = self.aria_label.or_else(|| label.clone());
        let binding = self.binding;
        let checked = binding
            .as_ref()
            .map_or(self.checked, |binding| binding.get(cx));
        let (box_bg, box_border) = if checked {
            (theme.accent, theme.accent)
        } else {
            (theme.panel, theme.border_strong)
        };
        let disabled = self.disabled;
        let handler = self.on_click;
        let interactive = !disabled && (binding.is_some() || handler.is_some());

        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap_2()
            .role(Role::CheckBox)
            .tab_index(0)
            .when_some(aria_label, |this, label| this.aria_label(label))
            .aria_toggled(Toggled::from(checked))
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
                    .rounded(px(radius::SM))
                    .border_1()
                    .border_color(box_border)
                    .bg(box_bg)
                    .when(checked, |this| {
                        this.child(
                            Icon::new(IconName::Check)
                                .size(IconSize::XSmall)
                                .color(theme.on_accent),
                        )
                    }),
            )
            .when_some(label, |this, label| {
                this.child(div().text_sm().text_color(theme.text).child(label))
            })
            .when(interactive, |this| {
                let binding_for_click = binding.clone();
                let binding_for_key = binding;
                let handler_for_click = handler.map(std::rc::Rc::new);
                let handler_for_key = handler_for_click.clone();
                this.on_click(move |event, window, cx| {
                    if let Some(binding) = &binding_for_click {
                        binding.update(cx, |checked| {
                            *checked = !*checked;
                            true
                        });
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
                        if let Some(binding) = &binding_for_key {
                            binding.update(cx, |checked| {
                                *checked = !*checked;
                                true
                            });
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
    fn bound_checkbox_stores_binding() {
        let mut app = TestApp::new();
        let checkbox = app.update(|cx| Checkbox::bound("checkbox", cx.binding(false)));

        assert!(checkbox.binding.is_some());
    }

    #[test]
    fn checkbox_can_store_explicit_aria_label() {
        let checkbox = Checkbox::new("checkbox", false).aria_label("Toggle notifications");

        assert_eq!(checkbox.aria_label.as_deref(), Some("Toggle notifications"));
    }
}
