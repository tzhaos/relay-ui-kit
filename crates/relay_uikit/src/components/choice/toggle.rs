use std::rc::Rc;

use gpui::{
    App, ClickEvent, ElementId, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, RenderOnce, Role, StatefulInteractiveElement, Styled, Toggled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    interaction::ClickHandler,
    theme::{ActiveTheme, DISABLED_OPACITY, space},
};

/// A sliding on/off switch. The host owns `on` and flips it in `on_click`.
#[derive(IntoElement)]
pub struct Toggle {
    id: ElementId,
    on: bool,
    label: Option<String>,
    disabled: bool,
    binding: Option<Binding<bool>>,
    on_click: Option<ClickHandler>,
}

impl Toggle {
    pub fn new(id: impl Into<ElementId>, on: bool) -> Self {
        Self {
            id: id.into(),
            on,
            label: None,
            disabled: false,
            binding: None,
            on_click: None,
        }
    }

    pub fn bound(id: impl Into<ElementId>, binding: Binding<bool>) -> Self {
        Self {
            id: id.into(),
            on: false,
            label: None,
            disabled: false,
            binding: Some(binding),
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

impl RenderOnce for Toggle {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let binding = self.binding;
        let on = binding.as_ref().map_or(self.on, |binding| binding.get(cx));
        let track_bg = if on {
            theme.accent
        } else {
            theme.border_strong
        };
        let disabled = self.disabled;
        let handler_rc = self.on_click.map(Rc::new);
        let interactive = !disabled && (binding.is_some() || handler_rc.is_some());

        let track = div()
            .w(px(32.0))
            .h(px(18.0))
            .flex_shrink_0()
            .rounded(px(9.0))
            .bg(track_bg)
            .p(px(space::XXS))
            .flex()
            .items_center()
            .when(on, |this| this.justify_end())
            .when(!on, |this| this.justify_start())
            .child(div().size(px(14.0)).rounded(px(7.0)).bg(theme.panel));

        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap_2()
            .role(Role::Switch)
            .tab_index(0)
            .aria_toggled(Toggled::from(on))
            .when(disabled, |this| this.opacity(DISABLED_OPACITY))
            .when(interactive, |this| {
                this.cursor_pointer()
                    .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                        window.prevent_default();
                    })
            })
            .child(track)
            .when_some(self.label, |this, label| {
                this.child(div().text_sm().text_color(theme.text).child(label))
            })
            .when(interactive, |this| {
                let binding_for_click = binding.clone();
                let binding_for_key = binding;
                let handler_for_click = handler_rc.clone();
                let handler_for_key = handler_rc;
                this.on_click(move |event, window, cx| {
                    if let Some(binding) = &binding_for_click {
                        binding.update(cx, |on| {
                            *on = !*on;
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
                            binding.update(cx, |on| {
                                *on = !*on;
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
    fn bound_toggle_stores_binding() {
        let mut app = TestApp::new();
        let toggle = app.update(|cx| Toggle::bound("toggle", cx.binding(false)));

        assert!(toggle.binding.is_some());
    }
}
