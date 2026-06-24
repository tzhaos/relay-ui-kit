//! Terminal-style content framing for logs, PTY projections, and command output panes.

use gpui::{
    AnyElement, App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, Theme, mono_family, space};

/// A terminal frame that hosts a real terminal/PTY projection.
///
/// Use this as the outer shell around [`crate::OutputLog`], a terminal view, or
/// any other scrollable command-output surface. Connectivity stays explicit so
/// the host can show stale output while marking the underlying session as
/// disconnected.
#[derive(IntoElement)]
pub struct OutputSurface {
    id: ElementId,
    content: Option<AnyElement>,
    connected: bool,
}

impl OutputSurface {
    /// Create a connected output surface with scrollable content.
    pub fn new(id: impl Into<ElementId>, content: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            content: Some(content.into_any_element()),
            connected: true,
        }
    }

    /// Create an empty, disconnected output surface placeholder.
    pub fn empty(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            content: None,
            connected: false,
        }
    }

    /// Override whether the session behind the surface is currently connected.
    pub fn connected(mut self, connected: bool) -> Self {
        self.connected = connected;
        self
    }
}

impl RenderOnce for OutputSurface {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();
        let id = self.id;
        let scroll_id = (id.clone(), "scroll");
        let connected = self.connected;
        let empty = self.content.is_none();
        let content = self.content;

        div()
            .size_full()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .id(id)
            .overflow_hidden()
            .bg(theme.terminal_bg)
            .font_family(mono_family())
            .text_size(px(13.0))
            .relative()
            .when_some(content, |this, content| {
                this.child(
                    div()
                        .id(scroll_id)
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
            .when(!connected && !empty, |this| {
                this.child(
                    div()
                        .absolute()
                        .inset_0()
                        .bg(gpui::red().opacity(0.08))
                        .occlude()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded(px(4.0))
                                .bg(gpui::red().opacity(0.5))
                                .text_xs()
                                .text_color(gpui::white())
                                .child("Session disconnected"),
                        ),
                )
            })
            .occlude()
    }
}

fn empty_terminal_state(theme: Theme, connected: bool) -> gpui::Div {
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
        let surface = OutputSurface::empty("terminal-empty");

        assert!(!surface.connected);
    }

    #[test]
    fn output_surface_accepts_owned_element_ids() {
        let surface = OutputSurface::empty(format!("terminal-{}", "review"));

        assert_eq!(surface.id, ElementId::Name("terminal-review".into()));
    }
}
