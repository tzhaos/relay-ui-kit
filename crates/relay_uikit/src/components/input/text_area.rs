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
    platform_input::{PointerSelectionState, SingleLineInputStyle, multiline_input},
};

/// A multi-line text area that can be host-owned or Relay-bound.
#[derive(IntoElement)]
pub struct TextArea {
    id: ElementId,
    focus: FocusHandle,
    before: String,
    after: String,
    placeholder: String,
    focused: bool,
    disabled: bool,
    min_rows: usize,
    bordered: bool,
    key_context: String,
    binding: Option<Binding<TextInputState>>,
    on_key: Option<KeyHandler>,
}

impl TextArea {
    /// Create a host-owned text area from a snapshot of [`TextInputState`].
    pub fn new(id: impl Into<ElementId>, focus: FocusHandle, state: &TextInputState) -> Self {
        let (before, after) = state.split();
        Self {
            id: id.into(),
            focus,
            before: before.to_string(),
            after: after.to_string(),
            placeholder: String::new(),
            focused: false,
            disabled: false,
            min_rows: 3,
            bordered: true,
            key_context: "TextArea".into(),
            binding: None,
            on_key: None,
        }
    }

    /// Create a Relay-bound text area backed by a [`Binding<TextInputState>`].
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
            focused: false,
            disabled: false,
            min_rows: 3,
            bordered: true,
            key_context: "TextArea".into(),
            binding: Some(binding),
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
        let pointer = binding.as_ref().map(|_| {
            window.use_keyed_state((root_id.clone(), "pointer-selection"), cx, |_, _| {
                PointerSelectionState::default()
            })
        });
        let (before, selection, after, _cursor_visible) = match binding.as_ref() {
            Some(binding) => {
                let state = binding.get(cx);
                let sel_range = state.selection_range();
                let value = state.value().to_string();
                if let Some((start, end)) = sel_range {
                    (
                        value[..start].to_string(),
                        value[start..end].to_string(),
                        value[end..].to_string(),
                        false,
                    )
                } else {
                    let (before, after) = state.split();
                    (before.to_string(), String::new(), after.to_string(), true)
                }
            }
            None => (self.before, String::new(), self.after, false),
        };
        let has_selection = !selection.is_empty();
        let border = if is_focused {
            theme.accent
        } else {
            theme.border_strong
        };
        let focus_for_click = self.focus.clone();
        let focus_for_mouse_down = self.focus.clone();
        let on_key = self.on_key;
        let handle_key = !self.disabled && (binding.is_some() || on_key.is_some());
        let show_placeholder =
            before.is_empty() && after.is_empty() && !has_selection && !is_focused;
        let min_height = self.min_rows as f32 * 20.0 + 16.0;
        let disabled = self.disabled;

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
                    .when(show_placeholder, |this| {
                        if let (Some(binding), Some(pointer)) = (binding.clone(), pointer.clone()) {
                            this.child(multiline_input(
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
                                self.min_rows,
                            ))
                        } else {
                            this.text_color(theme.text_muted).child(placeholder.clone())
                        }
                    })
                    .when(!show_placeholder, |this| {
                        if let (Some(binding), Some(pointer)) = (binding.clone(), pointer.clone()) {
                            this.child(multiline_input(
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
                                self.min_rows,
                            ))
                        } else {
                            this.text_color(theme.text).children(text_area_lines(
                                before,
                                after,
                                is_focused,
                                theme.accent,
                            ))
                        }
                    }),
            )
            .when(handle_key, |this| {
                this.on_key_down(move |event, window, cx| {
                    let mut consumed = false;
                    if let Some(binding) = &binding {
                        binding.update(cx, |state| {
                            let action = state.handle_platform_multiline_key(event);
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
                this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    window.focus(&focus_for_mouse_down, cx);
                })
                .on_click(move |_, window, cx| {
                    window.focus(&focus_for_click, cx);
                })
            })
    }
}

fn text_area_lines(
    before: String,
    after: String,
    focused: bool,
    caret_color: gpui::Hsla,
) -> Vec<gpui::AnyElement> {
    let mut before_lines = before.split('\n').map(str::to_string).collect::<Vec<_>>();
    let mut after_lines = after.split('\n').map(str::to_string).collect::<Vec<_>>();
    let cursor_prefix = before_lines.pop().unwrap_or_default();
    let cursor_suffix = after_lines.first().cloned().unwrap_or_default();
    if !after_lines.is_empty() {
        after_lines.remove(0);
    }

    let mut lines = Vec::with_capacity(before_lines.len() + after_lines.len() + 1);
    lines.extend(before_lines.into_iter().map(line_element));
    lines.push(
        div()
            .min_h(px(20.0))
            .flex()
            .items_center()
            .child(cursor_prefix)
            .when(focused, |this| this.child(caret(caret_color)))
            .child(cursor_suffix)
            .into_any_element(),
    );
    lines.extend(after_lines.into_iter().map(line_element));
    lines
}

fn line_element(line: String) -> gpui::AnyElement {
    div().min_h(px(20.0)).child(line).into_any_element()
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
    fn bound_text_area_stores_binding() {
        let mut app = TestApp::new();
        let area = app.update(|cx| {
            TextArea::bound(
                "area",
                cx.focus_handle(),
                cx.binding(TextInputState::with_text("relay")),
            )
        });

        assert!(area.binding.is_some());
    }

    #[test]
    fn key_context_accepts_owned_strings() {
        let mut app = TestApp::new();
        let area = app.update(|cx| {
            TextArea::new("area", cx.focus_handle(), &TextInputState::new())
                .key_context(format!("TextArea section={}", "notes"))
        });

        assert_eq!(area.key_context, "TextArea section=notes");
    }
}
