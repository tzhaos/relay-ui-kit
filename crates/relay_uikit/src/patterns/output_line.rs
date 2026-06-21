use crate::Theme;
use gpui::Hsla;

/// Visual treatment for one terminal output row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputLineStyle {
    Prompt,
    Input,
    Output,
    Muted,
    Success,
    Error,
}

impl OutputLineStyle {
    pub(crate) fn color(self, theme: &Theme) -> Hsla {
        match self {
            OutputLineStyle::Prompt | OutputLineStyle::Muted => theme.terminal_dim,
            OutputLineStyle::Input | OutputLineStyle::Output => theme.terminal_text,
            OutputLineStyle::Success => theme.accent,
            OutputLineStyle::Error => theme.danger,
        }
    }
}

/// A single row of text inside [`crate::TerminalTranscript`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputLine {
    pub text: String,
    pub style: OutputLineStyle,
}

impl OutputLine {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: OutputLineStyle::Output,
        }
    }

    pub fn style(mut self, style: OutputLineStyle) -> Self {
        self.style = style;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_line_defaults_to_output() {
        let line = OutputLine::new("cargo build");
        assert_eq!(line.style, OutputLineStyle::Output);
    }

    #[test]
    fn terminal_line_style_builder_sets_style() {
        let line = OutputLine::new("error").style(OutputLineStyle::Error);
        assert_eq!(line.style, OutputLineStyle::Error);
    }
}
