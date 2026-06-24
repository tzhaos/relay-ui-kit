//! Single-line text entry for Relay desktop surfaces.
//!
//! [`TextInput`] always renders against a live [`relay::Binding`] of
//! [`TextInputState`] so desktop selection, composition, and IME behavior stay
//! on the real Relay editing path.

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
        PlatformInputBase, PlatformInputMode, PointerSelectionState, SingleLineInputProps,
        SingleLineInputStyle, single_line_input,
    },
};

/// A single-line text input with Relay-bound desktop editing semantics.
#[derive(IntoElement)]
pub struct TextInput {
    id: ElementId,
    focus: FocusHandle,
    placeholder: String,
    leading_icon: Option<IconName>,
    focused: bool,
    disabled: bool,
    key_context: String,
    binding: Binding<TextInputState>,
    on_key: Option<KeyHandler>,
}

impl TextInput {
    /// Create a Relay-bound text input backed by a [`Binding<TextInputState>`].
    pub fn bound(
        id: impl Into<ElementId>,
        focus: FocusHandle,
        binding: Binding<TextInputState>,
    ) -> Self {
        Self {
            id: id.into(),
            focus,
            placeholder: String::new(),
            leading_icon: None,
            focused: false,
            disabled: false,
            key_context: "TextInput".into(),
            binding,
            on_key: None,
        }
    }

    /// Render placeholder text when the current value is empty.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Show a leading icon inside the input chrome.
    pub fn leading_icon(mut self, icon: IconName) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    /// Force the focused visual treatment without moving actual GPUI focus.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Disable the input — blocks keyboard input and click-to-focus.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Override the GPUI key-dispatch context used while this input is focused.
    ///
    /// This accepts owned strings so higher-level product surfaces can derive
    /// contexts from runtime ids instead of baking in static labels.
    pub fn key_context(mut self, key_context: impl Into<String>) -> Self {
        self.key_context = key_context.into();
        self
    }

    /// Observe raw `keydown` events after Relay's built-in editing behavior runs.
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
        let pointer = window.use_keyed_state((root_id.clone(), "pointer-selection"), cx, |_, _| {
            PointerSelectionState::default()
        });
        let show_placeholder = binding.get(cx).is_empty() && !is_focused;
        let border = if is_focused {
            theme.accent
        } else {
            theme.border_strong
        };
        let focus_for_click = self.focus.clone();
        let focus_for_mousedown = self.focus.clone();
        let on_key = self.on_key;
        let handle_key = !self.disabled;
        let disabled = self.disabled;
        let editor_style = SingleLineInputStyle {
            text_color: theme.text,
            placeholder_color: theme.text_muted,
            selection_color: theme.selection,
            cursor_color: theme.accent,
        };

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
            .key_context(self.key_context.as_str())
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
                    .child(single_line_input(SingleLineInputProps {
                        base: PlatformInputBase {
                            id: (root_id.clone(), "editor").into(),
                            focus: focus.clone(),
                            binding: binding.clone(),
                            pointer,
                            style: editor_style,
                            placeholder: placeholder.clone(),
                            show_placeholder,
                            disabled,
                        },
                        mode: PlatformInputMode::Text,
                        after_edit: None,
                    })),
            )
            .when(handle_key, |this| {
                this.on_key_down(move |event, window, cx| {
                    let mut consumed = false;
                    binding.update(cx, |state| {
                        let action = state.handle_platform_key(event);
                        consumed = action.should_notify();
                        consumed
                    });
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

        let value = app.read(|cx| input.binding.get(cx).value().to_string());
        assert_eq!(value, "relay");
    }

    #[test]
    fn key_context_accepts_owned_strings() {
        let mut app = TestApp::new();
        let input = app.update(|cx| {
            TextInput::bound("text", cx.focus_handle(), cx.binding(TextInputState::new()))
                .key_context(format!("TextInput mode={}", "search"))
        });

        assert_eq!(input.key_context, "TextInput mode=search");
    }
}
