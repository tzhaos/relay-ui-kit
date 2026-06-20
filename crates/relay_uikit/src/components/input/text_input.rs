use gpui::{
    App, ElementId, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, ParentElement,
    RenderOnce, Role, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::KeyHandler,
    theme::{ActiveTheme, radius},
};

use super::TextInputState;

/// A single-line text input view. The host owns the editable state.
#[derive(IntoElement)]
pub struct TextInput {
    id: ElementId,
    focus: FocusHandle,
    before: String,
    after: String,
    placeholder: String,
    leading_icon: Option<IconName>,
    focused: bool,
    key_context: &'static str,
    binding: Option<Binding<TextInputState>>,
    on_key: Option<KeyHandler>,
}

impl TextInput {
    pub fn new(id: impl Into<ElementId>, focus: FocusHandle, state: &TextInputState) -> Self {
        let (before, after) = state.split();
        Self {
            id: id.into(),
            focus,
            before: before.to_string(),
            after: after.to_string(),
            placeholder: String::new(),
            leading_icon: None,
            focused: false,
            key_context: "TextInput",
            binding: None,
            on_key: None,
        }
    }

    pub fn bound(
        id: impl Into<ElementId>,
        focus: FocusHandle,
        binding: Binding<TextInputState>,
    ) -> Self {
        Self {
            id: id.into(),
            focus,
            before: String::new(),
            after: String::new(),
            placeholder: String::new(),
            leading_icon: None,
            focused: false,
            key_context: "TextInput",
            binding: Some(binding),
            on_key: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn leading_icon(mut self, icon: IconName) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn key_context(mut self, key_context: &'static str) -> Self {
        self.key_context = key_context;
        self
    }

    pub fn on_key(
        mut self,
        handler: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_key = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for TextInput {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let binding = self.binding;
        let (before, after) = match binding.as_ref() {
            Some(binding) => {
                let state = binding.get(cx);
                let (before, after) = state.split();
                (before.to_string(), after.to_string())
            }
            None => (self.before, self.after),
        };
        let border = if self.focused {
            theme.accent
        } else {
            theme.border_strong
        };
        let focus_for_click = self.focus.clone();
        let on_key = self.on_key;
        let handle_key = binding.is_some() || on_key.is_some();
        let show_placeholder = before.is_empty() && after.is_empty() && !self.focused;

        div()
            .id(self.id)
            .h(px(30.0))
            .w_full()
            .flex()
            .items_center()
            .gap_2()
            .px_2()
            .rounded(px(radius::MD))
            .bg(theme.panel)
            .border_1()
            .border_color(border)
            .track_focus(&self.focus)
            .tab_index(0)
            .role(Role::TextInput)
            .key_context(self.key_context)
            .cursor(gpui::CursorStyle::IBeam)
            .when(!self.focused, |this| {
                this.hover(move |s| s.border_color(theme.border_strong))
            })
            .when_some(self.leading_icon, |this, icon| {
                this.child(
                    Icon::new(icon)
                        .size(IconSize::Small)
                        .color(theme.text_muted),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .text_sm()
                    .when(show_placeholder, |this| {
                        this.text_color(theme.text_muted).child(self.placeholder)
                    })
                    .when(!show_placeholder, |this| {
                        this.text_color(theme.text)
                            .child(before)
                            .when(self.focused, |this| this.child(caret(theme.accent)))
                            .child(after)
                    }),
            )
            .when(handle_key, |this| {
                this.on_key_down(move |event, window, cx| {
                    if let Some(binding) = &binding {
                        binding.update(cx, |state| state.handle_key(event).should_notify());
                    }
                    if let Some(on_key) = &on_key {
                        on_key(event, window, cx);
                    }
                    cx.stop_propagation();
                })
            })
            .on_click(move |_, window, cx| {
                window.focus(&focus_for_click, cx);
            })
    }
}

fn caret(color: gpui::Hsla) -> impl IntoElement {
    div().w(px(1.5)).h(px(16.0)).bg(color)
}

#[cfg(test)]
mod tests {
    use gpui::TestApp;
    use relay::ReactiveAppExt;

    use super::*;

    #[test]
    fn bound_text_input_stores_binding() {
        let mut app = TestApp::new();
        let input = app.update(|cx| {
            TextInput::bound(
                "text",
                cx.focus_handle(),
                cx.binding(TextInputState::with_text("relay")),
            )
        });

        assert!(input.binding.is_some());
    }
}
