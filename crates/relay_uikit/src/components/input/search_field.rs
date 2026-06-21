use gpui::{
    App, ClickEvent, ElementId, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use relay::Binding;

use crate::{
    icon::{Icon, IconName, IconSize},
    interaction::{ClickHandler, KeyCaptureHandler},
    theme::{ActiveTheme, DISABLED_OPACITY, radius},
};

use super::TextInputState;

/// A focusable search/filter well with a leading magnifier icon.
#[derive(IntoElement)]
pub struct SearchField {
    id: ElementId,
    focus: FocusHandle,
    value: String,
    placeholder: String,
    disabled: bool,
    key_context: &'static str,
    binding: Option<Binding<TextInputState>>,
    on_key: Option<KeyCaptureHandler>,
    on_clear: Option<ClickHandler>,
}

impl SearchField {
    pub fn new(id: impl Into<ElementId>, focus: FocusHandle) -> Self {
        Self {
            id: id.into(),
            focus,
            value: String::new(),
            placeholder: "Search".into(),
            disabled: false,
            key_context: "SearchField",
            binding: None,
            on_key: None,
            on_clear: None,
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
            value: String::new(),
            placeholder: "Search".into(),
            disabled: false,
            key_context: "SearchField",
            binding: Some(binding),
            on_key: None,
            on_clear: None,
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Disable the search field — blocks keyboard input and click-to-focus.
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
        handler: impl Fn(&KeyDownEvent, &mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.on_key = Some(Box::new(handler));
        self
    }

    pub fn on_clear(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_clear = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SearchField {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let binding = self.binding;
        let value = binding.as_ref().map_or_else(
            || self.value,
            |binding| binding.read(cx, |state| state.value().to_string()),
        );
        let is_empty = value.is_empty();
        let display = if is_empty {
            self.placeholder.clone()
        } else {
            value
        };
        let text_color = if is_empty {
            theme.text_muted
        } else {
            theme.text
        };
        let focus_for_click = self.focus.clone();
        let focus_for_mouse_down = self.focus.clone();
        let on_key = self.on_key;
        let handle_key = !self.disabled && (binding.is_some() || on_key.is_some());
        let disabled = self.disabled;
        let on_clear = self.on_clear;

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
            .border_color(theme.border)
            .track_focus(&self.focus)
            .tab_index(0)
            .key_context(self.key_context)
            .when(disabled, |this| this.opacity(DISABLED_OPACITY).cursor(gpui::CursorStyle::OperationNotAllowed))
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
                    .truncate()
                    .text_sm()
                    .text_color(text_color)
                    .child(display),
            )
            .when(!is_empty && !disabled, |this| {
                let on_clear_for_click = on_clear.map(std::rc::Rc::new);
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
                this.on_key_down(move |event, window, cx| {
                    let mut consumed = false;
                    if let Some(binding) = &binding {
                        binding.update(cx, |state| {
                            let action = state.handle_key(event);
                            consumed = action.should_notify();
                            consumed
                        });
                    }
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

#[cfg(test)]
mod tests {
    use gpui::TestApp;
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

        assert!(field.binding.is_some());
    }
}
