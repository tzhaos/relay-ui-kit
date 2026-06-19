use gpui::{
    App, ElementId, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, radius};

use super::TextInputState;

type KeyHandler = Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;

/// A multi-line text area view. The host owns the editable state.
#[derive(IntoElement)]
pub struct TextArea {
    id: ElementId,
    focus: FocusHandle,
    before: String,
    after: String,
    is_empty: bool,
    placeholder: String,
    focused: bool,
    min_rows: usize,
    key_context: &'static str,
    on_key: Option<KeyHandler>,
}

impl TextArea {
    pub fn new(id: impl Into<ElementId>, focus: FocusHandle, state: &TextInputState) -> Self {
        let (before, after) = state.split();
        Self {
            id: id.into(),
            focus,
            before: before.to_string(),
            after: after.to_string(),
            is_empty: state.is_empty(),
            placeholder: String::new(),
            focused: false,
            min_rows: 3,
            key_context: "TextArea",
            on_key: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn min_rows(mut self, rows: usize) -> Self {
        self.min_rows = rows.max(2);
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

impl RenderOnce for TextArea {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let border = if self.focused {
            theme.accent
        } else {
            theme.border_strong
        };
        let focus_for_click = self.focus.clone();
        let on_key = self.on_key;
        let show_placeholder = self.is_empty && !self.focused;
        let min_height = self.min_rows as f32 * 20.0 + 16.0;

        div()
            .id(self.id)
            .min_h(px(min_height))
            .w_full()
            .p_2()
            .rounded(px(radius::MD))
            .bg(theme.panel)
            .border_1()
            .border_color(border)
            .track_focus(&self.focus)
            .tab_index(0)
            .key_context(self.key_context)
            .cursor(gpui::CursorStyle::IBeam)
            .when(!self.focused, |this| {
                this.hover(move |s| s.border_color(theme.border_strong))
            })
            .child(
                div()
                    .w_full()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .text_sm()
                    .line_height(px(20.0))
                    .when(show_placeholder, |this| {
                        this.text_color(theme.text_muted).child(self.placeholder)
                    })
                    .when(!show_placeholder, |this| {
                        this.text_color(theme.text).children(text_area_lines(
                            self.before,
                            self.after,
                            self.focused,
                            theme.accent,
                        ))
                    }),
            )
            .when_some(on_key, |this, on_key| {
                this.on_key_down(move |event, window, cx| {
                    on_key(event, window, cx);
                    cx.stop_propagation();
                })
            })
            .on_click(move |_, window, _| {
                window.focus(&focus_for_click);
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
