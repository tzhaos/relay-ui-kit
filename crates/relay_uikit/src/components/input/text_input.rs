use gpui::{
    App, ElementId, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, RenderOnce, Role, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::KeyHandler,
    theme::{ActiveTheme, DISABLED_OPACITY, radius},
};

use super::{
    TextInputState,
    platform_input::{
        PlatformInputMode, PointerSelectionState, SingleLineInputStyle, single_line_input,
    },
};

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
    disabled: bool,
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
            disabled: false,
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
            disabled: false,
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

    /// Disable the input — blocks keyboard input and click-to-focus.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let binding = self.binding;
        let is_focused = self.focus.is_focused(window) || self.focused;
        let root_id = self.id.clone();
        let placeholder = self.placeholder.clone();
        let focus = self.focus.clone();
        let pointer = binding.as_ref().map(|_| {
            window.use_keyed_state((root_id.clone(), "pointer-selection"), cx, |_, _| {
                PointerSelectionState::default()
            })
        });
        let (before_str, selection_str, after_str, cursor_visible) = match binding.as_ref() {
            Some(binding) => {
                let state = binding.get(cx);
                let sel_range = state.selection_range();
                let value = state.value().to_string();
                if let Some((start, end)) = sel_range {
                    let before = value[..start].to_string();
                    let selected = value[start..end].to_string();
                    let after = value[end..].to_string();
                    (before, selected, after, false)
                } else {
                    let (before, after) = state.split();
                    (before.to_string(), String::new(), after.to_string(), true)
                }
            }
            None => (self.before, String::new(), self.after, false),
        };
        let border = if is_focused {
            theme.accent
        } else {
            theme.border_strong
        };
        let focus_for_click = self.focus.clone();
        let focus_for_mousedown = self.focus.clone();
        let on_key = self.on_key;
        let handle_key = !self.disabled && (binding.is_some() || on_key.is_some());
        let show_placeholder = before_str.is_empty()
            && after_str.is_empty()
            && selection_str.is_empty()
            && !is_focused;
        let disabled = self.disabled;

        div()
            .id(root_id.clone())
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
            .when(disabled, |this| {
                this.opacity(DISABLED_OPACITY)
                    .cursor(gpui::CursorStyle::OperationNotAllowed)
            })
            .when(!disabled, |this| {
                this.cursor(gpui::CursorStyle::IBeam)
                    .when(!is_focused, |this| {
                        this.hover(move |s| s.border_color(theme.border_strong))
                    })
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
                    .relative()
                    .flex()
                    .items_center()
                    .text_sm()
                    .when(show_placeholder, |this| {
                        if let (Some(binding), Some(pointer)) = (binding.clone(), pointer.clone()) {
                            this.child(single_line_input(
                                (root_id.clone(), "editor"),
                                focus.clone(),
                                binding,
                                pointer,
                                SingleLineInputStyle {
                                    text_color: theme.text,
                                    placeholder_color: theme.text_muted,
                                    selection_color: theme.selection,
                                    cursor_color: theme.accent,
                                },
                                placeholder.clone(),
                                show_placeholder,
                                disabled,
                                PlatformInputMode::Text,
                                None,
                            ))
                        } else {
                            this.text_color(theme.text_muted).child(placeholder.clone())
                        }
                    })
                    .when(!show_placeholder, |this| {
                        if let (Some(binding), Some(pointer)) = (binding.clone(), pointer.clone()) {
                            this.child(single_line_input(
                                (root_id.clone(), "editor"),
                                focus.clone(),
                                binding,
                                pointer,
                                SingleLineInputStyle {
                                    text_color: theme.text,
                                    placeholder_color: theme.text_muted,
                                    selection_color: theme.selection,
                                    cursor_color: theme.accent,
                                },
                                placeholder.clone(),
                                show_placeholder,
                                disabled,
                                PlatformInputMode::Text,
                                None,
                            ))
                        } else {
                            this.text_color(theme.text)
                                .child(div().child(before_str))
                                .when(!selection_str.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .bg(theme.selection)
                                            .text_color(theme.text)
                                            .child(selection_str),
                                    )
                                })
                                .when(is_focused && cursor_visible, |this| {
                                    this.child(caret(theme.accent))
                                })
                                .child(div().child(after_str))
                        }
                    }),
            )
            .when(handle_key, |this| {
                let binding_clone = binding.clone();
                this.on_key_down(move |event, window, cx| {
                    let mut consumed = false;
                    if let Some(binding) = &binding_clone {
                        binding.update(cx, |state| {
                            let action = state.handle_platform_key(event);
                            consumed = action.should_notify();
                            consumed
                        });
                    }
                    if let Some(on_key) = &on_key {
                        on_key(event, window, cx);
                    }
                    if consumed {
                        cx.stop_propagation();
                    }
                })
            })
            .when(!disabled, |this| {
                this.on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                    window.focus(&focus_for_mousedown, cx);
                })
                .on_click(move |_, window, cx| {
                    window.focus(&focus_for_click, cx);
                })
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
