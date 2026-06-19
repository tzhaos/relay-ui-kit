use gpui::{
    AnyElement, App, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, mono_family, space};

/// A terminal frame that hosts a real terminal/PTY projection.
#[derive(IntoElement)]
pub struct TerminalSurface {
    id: &'static str,
    content: Option<AnyElement>,
    connected: bool,
}

impl TerminalSurface {
    pub fn new(id: &'static str, content: impl IntoElement) -> Self {
        Self {
            id,
            content: Some(content.into_any_element()),
            connected: true,
        }
    }

    pub fn empty(id: &'static str) -> Self {
        Self {
            id,
            content: None,
            connected: false,
        }
    }

    pub fn connected(mut self, connected: bool) -> Self {
        self.connected = connected;
        self
    }
}

impl RenderOnce for TerminalSurface {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let id = self.id;
        let connected = self.connected;
        let empty = self.content.is_none();
        let content = self.content;

        div()
            .flex_1()
            .min_h_0()
            .id(id)
            .overflow_hidden()
            .bg(theme.terminal_bg)
            .font_family(mono_family())
            .text_size(px(13.0))
            .when_some(content, |this, content| {
                this.child(
                    div()
                        .id((id, 0usize))
                        .size_full()
                        .min_h_0()
                        .overflow_y_scroll()
                        .p_3()
                        .child(content),
                )
            })
            .when(empty, |this| {
                this.items_center()
                    .justify_center()
                    .child(empty_terminal_state(theme, connected))
            })
            .occlude()
    }
}

fn empty_terminal_state(theme: crate::Theme, connected: bool) -> gpui::Div {
    let title = if connected {
        "No terminal output"
    } else {
        "No terminal session"
    };
    let detail = if connected {
        "The session is attached and waiting for output."
    } else {
        "Open a project or create a terminal session."
    };

    div()
        .max_w(px(360.0))
        .flex()
        .flex_col()
        .items_center()
        .gap_1()
        .px(px(space::LG))
        .text_center()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.terminal_text)
                .child(title),
        )
        .child(div().text_xs().text_color(theme.terminal_dim).child(detail))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_surface_empty_starts_disconnected() {
        let surface = TerminalSurface::empty("terminal-empty");

        assert!(!surface.connected);
    }
}
