//! Multi-line text entry with Relay-aware selection, IME, and placeholder handling.

use gpui::{
    App, ElementId, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    interaction::KeyHandler,
    theme::{ActiveTheme, DISABLED_OPACITY, radius},
};

use super::{
    TextInputState,
    platform_input::{
        MultilineInputProps, PlatformInputBase, PointerSelectionState, SingleLineInputStyle,
        multiline_input,
    },
};

/// A multi-line text area with Relay-bound desktop editing semantics.
///
/// `TextArea` always renders against a live [`Binding<TextInputState>`] so
/// multiline selection, composition, and IME behavior remain on the real Relay
/// editing path.
#[derive(IntoElement)]
pub struct TextArea {
    id: ElementId,
    focus: FocusHandle,
    placeholder: String,
    focused: bool,
    disabled: bool,
    min_rows: usize,
    bordered: bool,
    key_context: String,
    binding: Binding<TextInputState>,
    on_key: Option<KeyHandler>,
}

impl TextArea {
    /// Create a Relay-bound text area backed by a [`Binding<TextInputState>`].
    pub fn bound(
        id: impl Into<ElementId>,
        focus: FocusHandle,
        binding: Binding<TextInputState>,
    ) -> Self {
        Self {
            id: id.into(),
            focus,
            placeholder: String::new(),
            focused: false,
            disabled: false,
            min_rows: 3,
            bordered: true,
            key_context: "TextArea".into(),
            binding,
            on_key: None,
        }
    }

    /// Render placeholder text when the current value is empty and unfocused.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Force the focused visual treatment without moving actual GPUI focus.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Disable the text area — blocks keyboard input and click-to-focus.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Guarantee a minimum visible line count for the editor.
    pub fn min_rows(mut self, rows: usize) -> Self {
        self.min_rows = rows.max(2);
        self
    }

    /// Toggle the outer field chrome while keeping text layout unchanged.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// Override the GPUI key-dispatch context used while this text area is focused.
    ///
    /// This accepts owned strings so app surfaces can scope multiline shortcuts
    /// with runtime-derived context keys.
    pub fn key_context(mut self, key_context: impl Into<String>) -> Self {
        self.key_context = key_context.into();
        self
    }

    /// Observe raw `keydown` events after Relay's built-in multiline editing behavior runs.
    pub fn on_key(
        mut self,
        handler: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_key = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for TextArea {
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
        let focus_for_mouse_down = self.focus.clone();
        let on_key = self.on_key;
        let handle_key = !self.disabled;
        let min_height = self.min_rows as f32 * 20.0 + 16.0;
        let disabled = self.disabled;
        let editor_style = SingleLineInputStyle {
            text_color: theme.text,
            placeholder_color: theme.text_muted,
            selection_color: theme.selection,
            cursor_color: theme.accent,
        };

        div()
            .id(root_id.clone())
            .min_h(px(min_height))
            .w_full()
            .p_2()
            .rounded(px(radius::MD))
            .when(self.bordered, |this| {
                this.bg(theme.panel).border_1().border_color(border)
            })
            .when(!self.bordered, |this| this.bg(gpui::transparent_black()))
            .track_focus(&self.focus)
            .tab_index(0)
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
            .child(
                div()
                    .w_full()
                    .min_h_0()
                    .relative()
                    .text_sm()
                    .line_height(px(20.0))
                    .child(multiline_input(MultilineInputProps {
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
                        min_rows: self.min_rows,
                    })),
            )
            .when(handle_key, |this| {
                this.on_key_down(move |event, window, cx| {
                    let mut consumed = false;
                    binding.update(cx, |state| {
                        consumed = state.handle_platform_multiline_key(event).should_notify();
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
                this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    window.focus(&focus_for_mouse_down, cx);
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
    fn bound_text_area_stores_binding() {
        let mut app = TestApp::new();
        let area = app.update(|cx| {
            TextArea::bound(
                "area",
                cx.focus_handle(),
                cx.binding(TextInputState::with_text("relay")),
            )
        });

        let value = app.read(|cx| area.binding.get(cx).value().to_string());
        assert_eq!(value, "relay");
    }

    #[test]
    fn key_context_accepts_owned_strings() {
        let mut app = TestApp::new();
        let area = app.update(|cx| {
            TextArea::bound("area", cx.focus_handle(), cx.binding(TextInputState::new()))
                .key_context(format!("TextArea section={}", "notes"))
        });

        assert_eq!(area.key_context, "TextArea section=notes");
    }
}
