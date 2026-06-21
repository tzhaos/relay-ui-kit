use gpui::{
    App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, prelude::FluentBuilder, px,
};

use crate::theme::{ActiveTheme, Theme, space};

use crate::patterns::OutputLine;

/// A lightweight transcript renderer for examples and read-only logs.
#[derive(IntoElement)]
pub struct OutputLog {
    lines: Vec<OutputLine>,
    prompt: Option<String>,
    cursor: bool,
}

impl OutputLog {
    pub fn new(lines: Vec<OutputLine>) -> Self {
        Self {
            lines,
            prompt: None,
            cursor: true,
        }
    }

    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    pub fn cursor(mut self, cursor: bool) -> Self {
        self.cursor = cursor;
        self
    }
}

impl RenderOnce for OutputLog {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();

        div()
            .flex()
            .flex_col()
            .gap(px(space::XXS))
            .children(
                self.lines
                    .into_iter()
                    .map(|line| terminal_line(theme, line)),
            )
            .when_some(self.prompt, |this, prompt| {
                this.child(prompt_line(theme, prompt, self.cursor))
            })
    }
}

fn terminal_line(theme: Theme, line: OutputLine) -> gpui::Div {
    div()
        .min_h(px(18.0))
        .text_color(line.style.color(&theme))
        .child(line.text)
}

fn prompt_line(theme: Theme, prompt: String, cursor: bool) -> gpui::Div {
    div()
        .min_h(px(18.0))
        .flex()
        .items_center()
        .gap_1()
        .child(div().text_color(theme.terminal_dim).child(prompt))
        .when(cursor, |this| {
            this.child(div().w(px(8.0)).h(px(15.0)).bg(theme.terminal_text))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_transcript_starts_without_prompt() {
        let transcript = OutputLog::new(Vec::new());

        assert!(transcript.prompt.is_none());
    }
}
