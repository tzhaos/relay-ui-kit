//! Search-focused single-line text entry with built-in leading icon and clear affordance.

use gpui::{
    App, ClickEvent, ElementId, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, RenderOnce, Role, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{ClickHandler, KeyCaptureHandler},
    theme::{ActiveTheme, DISABLED_OPACITY, radius},
};

use super::{
    TextInputState,
    platform_input::{
        PlatformInputBase, PlatformInputMode, PointerSelectionState, SingleLineInputProps,
        SingleLineInputStyle, single_line_input,
    },
};

/// A focusable search field with a leading magnifier icon and optional clear action.
///
/// `SearchField` always renders against a live [`Binding<TextInputState>`] so
/// desktop text editing, selection, and IME behavior stay on the real Relay
/// editing path instead of a host-owned preview snapshot.
#[derive(IntoElement)]
pub struct SearchField {
    id: ElementId,
    focus: FocusHandle,
    placeholder: String,
    disabled: bool,
    key_context: String,
    binding: Binding<TextInputState>,
    on_key: Option<KeyCaptureHandler>,
    on_clear: Option<ClickHandler>,
}

impl SearchField {
    /// Create a Relay-bound search field backed by a [`Binding<TextInputState>`].
    pub fn bound(
        id: impl Into<ElementId>,
        focus: FocusHandle,
        binding: Binding<TextInputState>,
    ) -> Self {
        Self {
            id: id.into(),
            focus,
            placeholder: "Search".into(),
            disabled: false,
            key_context: "SearchField".into(),
            binding,
            on_key: None,
            on_clear: None,
        }
    }

    /// Override the placeholder copy shown when the field is empty.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Disable the search field — blocks keyboard input and click-to-focus.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Override the GPUI key-dispatch context used while this field is focused.
    ///
    /// This accepts owned strings so product surfaces can derive contexts from
    /// runtime ids or pane roles.
    pub fn key_context(mut self, key_context: impl Into<String>) -> Self {
        self.key_context = key_context.into();
        self
    }

    /// Intercept `keydown` events and report whether the host consumed them.
    pub fn on_key(
        mut self,
        handler: impl Fn(&KeyDownEvent, &mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.on_key = Some(Box::new(handler));
        self
    }

    /// Wire the trailing clear affordance.
    pub fn on_clear(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_clear = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SearchField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let binding = self.binding;
        let is_focused = self.focus.is_focused(window);
        let root_id = self.id.clone();
        let placeholder = self.placeholder.clone();
        let focus = self.focus.clone();
        let pointer = window.use_keyed_state((root_id.clone(), "pointer-selection"), cx, |_, _| {
            PointerSelectionState::default()
        });
        let is_empty = binding.get(cx).is_empty();
        let show_placeholder = is_empty && !is_focused;
        let focus_for_click = self.focus.clone();
        let focus_for_mouse_down = self.focus.clone();
        let focus_for_clear = self.focus.clone();
        let on_key = self.on_key;
        let handle_key = !self.disabled;
        let disabled = self.disabled;
        let on_clear = self.on_clear;
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
            .border_color(if is_focused {
                theme.accent
            } else {
                theme.border
            })
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
                    .hover(move |style| style.border_color(theme.border_strong))
            })
            .child(
                Icon::new(IconName::Search)
                    .size(IconSize::Small)
                    .color(theme.text_muted),
            )
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
            .when(!is_empty && !disabled, |this| {
                let on_clear_for_click = on_clear.map(std::rc::Rc::new);
                let binding_for_clear = binding.clone();
                this.child(
                    div()
                        .id("search-clear")
                        .size(px(18.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(radius::SM))
                        .cursor_pointer()
                        .hover(move |style| style.bg(theme.hover))
                        .on_mouse_down(MouseButton::Left, |_event, window, _cx| {
                            window.prevent_default();
                        })
                        .on_click(move |event, window, cx| {
                            clear_search_binding(&binding_for_clear, cx);
                            window.focus(&focus_for_clear, cx);
                            if let Some(handler) = &on_clear_for_click {
                                handler(event, window, cx);
                            }
                            cx.stop_propagation();
                        })
                        .child(
                            Icon::new(IconName::X)
                                .size(IconSize::XSmall)
                                .color(theme.text_muted),
                        ),
                )
            })
            .when(handle_key, |this| {
                let binding_for_key = binding.clone();
                this.on_key_down(move |event, window, cx| {
                    let mut consumed = handle_bound_search_key(&binding_for_key, event, cx);
                    if let Some(on_key) = &on_key
                        && on_key(event, window, cx)
                    {
                        consumed = true;
                    }
                    if consumed {
                        cx.stop_propagation();
                    }
                })
            })
            .when(!disabled, |this| {
                this.on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                    window.focus(&focus_for_mouse_down, cx);
                    window.prevent_default();
                })
                .on_click(move |_: &ClickEvent, window, cx| {
                    window.focus(&focus_for_click, cx);
                })
            })
    }
}

fn clear_search_binding(binding: &Binding<TextInputState>, cx: &mut App) -> bool {
    let mut cleared = false;
    binding.update(cx, |state| {
        if state.is_empty() {
            cleared = false;
            false
        } else {
            state.clear();
            cleared = true;
            true
        }
    });
    cleared
}

fn handle_bound_search_key(
    binding: &Binding<TextInputState>,
    event: &KeyDownEvent,
    cx: &mut App,
) -> bool {
    if event.keystroke.key == "escape" {
        return clear_search_binding(binding, cx);
    }

    let mut consumed = false;
    binding.update(cx, |state| {
        consumed = state.handle_platform_key(event).should_notify();
        consumed
    });
    consumed
}

#[cfg(test)]
mod tests {
    use gpui::{KeyDownEvent, Keystroke, Modifiers, TestApp};
    use relay::ReactiveAppExt;

    use super::*;

    #[test]
    fn bound_search_field_stores_binding() {
        let mut app = TestApp::new();
        let field = app.update(|cx| {
            SearchField::bound(
                "search",
                cx.focus_handle(),
                cx.binding(TextInputState::with_text("relay")),
            )
        });

        let value = app.read(|cx| field.binding.get(cx).value().to_string());
        assert_eq!(value, "relay");
    }

    #[test]
    fn key_context_accepts_owned_strings() {
        let mut app = TestApp::new();
        let field = app.update(|cx| {
            SearchField::bound(
                "search",
                cx.focus_handle(),
                cx.binding(TextInputState::new()),
            )
            .key_context(format!("SearchField pane={}", "left"))
        });

        assert_eq!(field.key_context, "SearchField pane=left");
    }

    #[test]
    fn clear_button_default_logic_clears_bound_state() {
        let mut app = TestApp::new();
        let binding = app.update(|cx| cx.binding(TextInputState::with_text("relay")));

        app.update(|cx| clear_search_binding(&binding, cx));

        let value = app.read(|cx| binding.get(cx).value().to_string());
        assert_eq!(value, "");
    }

    #[test]
    fn escape_clears_non_empty_bound_search() {
        let mut app = TestApp::new();
        let binding = app.update(|cx| cx.binding(TextInputState::with_text("relay")));

        let consumed = app.update(|cx| handle_bound_search_key(&binding, &key("escape", None), cx));
        let value = app.read(|cx| binding.get(cx).value().to_string());

        assert!(consumed);
        assert_eq!(value, "");
    }

    #[test]
    fn escape_bubbles_when_bound_search_is_already_empty() {
        let mut app = TestApp::new();
        let binding = app.update(|cx| cx.binding(TextInputState::new()));

        let consumed = app.update(|cx| handle_bound_search_key(&binding, &key("escape", None), cx));

        assert!(!consumed);
    }

    fn key(name: &str, ch: Option<&str>) -> KeyDownEvent {
        KeyDownEvent {
            keystroke: Keystroke {
                key: name.to_string(),
                key_char: ch.map(|value| value.to_string()),
                modifiers: Modifiers::default(),
            },
            is_held: false,
            prefer_character_input: false,
        }
    }
}
