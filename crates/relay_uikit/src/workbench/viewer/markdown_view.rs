use gpui::{
    App, FontWeight, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, px,
};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::theme::{ActiveTheme, BORDER_WIDTH, Theme, mono_family, radius};

/// An inline formatted span within a block.
#[derive(Debug, Clone, PartialEq, Eq)]
enum InlineSpan {
    Text(String),
    Bold(String),
    Italic(String),
    Strikethrough(String),
    Code(String),
    Link { text: String, url: String },
}

impl InlineSpan {
    fn render(&self, theme: Theme) -> gpui::Div {
        match self {
            InlineSpan::Text(text) => div().child(text.clone()),
            InlineSpan::Bold(text) => div().font_weight(FontWeight::BOLD).child(text.clone()),
            InlineSpan::Italic(text) => div().italic().child(text.clone()),
            InlineSpan::Strikethrough(text) => div().italic().opacity(0.6).child(text.clone()),
            InlineSpan::Code(text) => div()
                .px(px(2.0))
                .rounded(px(2.0))
                .bg(theme.inset)
                .font_family(mono_family())
                .text_size(px(11.0))
                .child(text.clone()),
            InlineSpan::Link { text, url: _ } => {
                div().text_color(theme.accent).underline().child(text.clone())
            }
        }
    }
}

fn spans_to_string(spans: &[InlineSpan]) -> String {
    spans
        .iter()
        .map(|s| match s {
            InlineSpan::Text(t)
            | InlineSpan::Bold(t)
            | InlineSpan::Italic(t)
            | InlineSpan::Strikethrough(t)
            | InlineSpan::Code(t) => t.clone(),
            InlineSpan::Link { text, .. } => text.clone(),
        })
        .collect::<Vec<_>>()
        .join("")
}

/// A content block extracted from Markdown.
#[derive(Debug, Clone, PartialEq, Eq)]
enum MarkdownBlock {
    Heading(u8, Vec<InlineSpan>),
    Paragraph(Vec<InlineSpan>),
    ListItem(Vec<InlineSpan>),
    Code(String),
    Quote(Vec<InlineSpan>),
    Rule,
}

/// A compact read-only Markdown preview for context panes.
#[derive(IntoElement)]
pub struct MarkdownView {
    blocks: Vec<MarkdownBlock>,
}

impl MarkdownView {
    /// Parses Markdown into a lightweight block model for display.
    pub fn new(source: impl AsRef<str>) -> Self {
        Self {
            blocks: parse_markdown(source.as_ref()),
        }
    }
}

impl RenderOnce for MarkdownView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.theme();

        div()
            .id("markdown-view")
            .size_full()
            .min_h_0()
            .overflow_y_scroll()
            .bg(theme.panel)
            .p_3()
            .flex()
            .flex_col()
            .gap_2()
            .children(
                self.blocks
                    .into_iter()
                    .map(move |block| render_block(theme, block)),
            )
    }
}

fn render_block(theme: Theme, block: MarkdownBlock) -> gpui::Div {
    match block {
        MarkdownBlock::Heading(level, spans) => div()
            .text_size(px(if level <= 1 { 16.0 } else { 14.0 }))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme.text)
            .children(spans.into_iter().map(move |s| s.render(theme))),
        MarkdownBlock::Paragraph(spans) => div()
            .text_sm()
            .text_color(theme.text_secondary)
            .children(spans.into_iter().map(move |s| s.render(theme))),
        MarkdownBlock::ListItem(spans) => div()
            .flex()
            .gap_2()
            .text_sm()
            .text_color(theme.text_secondary)
            .child(
                div()
                    .flex_shrink_0()
                    .text_color(theme.text_muted)
                    .child("-"),
            )
            .child(
                div()
                    .min_w_0()
                    .children(spans.into_iter().map(move |s| s.render(theme))),
            ),
        MarkdownBlock::Code(text) => div()
            .p_2()
            .rounded(px(radius::MD))
            .bg(theme.inset)
            .font_family(mono_family())
            .text_size(px(12.0))
            .text_color(theme.text_secondary)
            .child(text),
        MarkdownBlock::Quote(spans) => div()
            .pl_2()
            .border_l_1()
            .border_color(theme.border_strong)
            .text_sm()
            .text_color(theme.text_muted)
            .children(spans.into_iter().map(move |s| s.render(theme))),
        MarkdownBlock::Rule => div().h(px(BORDER_WIDTH)).bg(theme.border),
    }
}

/// Stack-based text accumulator for nested block structures.
#[derive(Default)]
struct Accumulator {
    /// Current accumulated inline spans.
    spans: Vec<InlineSpan>,
    /// Pending text buffer for building span content.
    buf: String,
    /// Current inline formatting mode.
    inline_mode: InlineMode,
    /// Pending link URL.
    link_url: Option<String>,
}

#[derive(Clone, Copy, Default)]
enum InlineMode {
    #[default]
    Plain,
    Bold,
    Italic,
    Strikethrough,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockKind {
    Heading(u8),
    Paragraph,
    ListItem,
    Code,
    Quote,
}

impl Accumulator {
    fn push_text(&mut self, text: &str) {
        self.buf.push_str(text);
    }

    fn flush_buf(&mut self) {
        if self.buf.is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.buf);
        let span = if let Some(url) = self.link_url.take() {
            InlineSpan::Link { text, url }
        } else {
            match self.inline_mode {
                InlineMode::Plain => InlineSpan::Text(text),
                InlineMode::Bold => InlineSpan::Bold(text),
                InlineMode::Italic => InlineSpan::Italic(text),
                InlineMode::Strikethrough => InlineSpan::Strikethrough(text),
            }
        };
        self.spans.push(span);
    }

    fn take_spans(&mut self) -> Vec<InlineSpan> {
        self.flush_buf();
        std::mem::take(&mut self.spans)
    }

    fn take_text(&mut self) -> String {
        self.flush_buf();
        let text = spans_to_string(&self.spans);
        self.spans.clear();
        text
    }

    fn clear(&mut self) {
        self.spans.clear();
        self.buf.clear();
    }
}

fn parse_markdown(source: &str) -> Vec<MarkdownBlock> {
    let parser = Parser::new_ext(source, Options::ENABLE_STRIKETHROUGH);
    let mut blocks = Vec::new();
    let mut stack: Vec<Accumulator> = vec![Accumulator::default()];
    let mut block_kind: Option<BlockKind> = None;
    let mut list_depth: usize = 0;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    stack.last_mut().unwrap().clear();
                    block_kind = Some(BlockKind::Heading(level_to_u8(level)));
                }
                Tag::List(_) => {
                    list_depth += 1;
                }
                Tag::Item => {
                    // Flush any pending paragraphs before a new list item,
                    // but not when already inside a list item (nested lists).
                    if block_kind != Some(BlockKind::ListItem) {
                        flush_block(&mut blocks, &mut stack, &mut block_kind);
                    }
                    stack.push(Accumulator::default());
                    block_kind = Some(BlockKind::ListItem);
                }
                Tag::Paragraph => {
                    // If we're inside a list item, paragraphs are inline content
                    if block_kind == Some(BlockKind::ListItem) {
                        // Continue accumulating within the list item
                    } else {
                        flush_block(&mut blocks, &mut stack, &mut block_kind);
                        block_kind = Some(BlockKind::Paragraph);
                    }
                }
                Tag::BlockQuote(_) => {
                    flush_block(&mut blocks, &mut stack, &mut block_kind);
                    stack.push(Accumulator::default());
                    block_kind = Some(BlockKind::Quote);
                }
                Tag::CodeBlock(_) => {
                    flush_block(&mut blocks, &mut stack, &mut block_kind);
                    stack.push(Accumulator::default());
                    block_kind = Some(BlockKind::Code);
                }
                Tag::Emphasis => {
                    stack.last_mut().unwrap().flush_buf();
                    stack.last_mut().unwrap().inline_mode = InlineMode::Italic;
                }
                Tag::Strong => {
                    stack.last_mut().unwrap().flush_buf();
                    stack.last_mut().unwrap().inline_mode = InlineMode::Bold;
                }
                Tag::Strikethrough => {
                    stack.last_mut().unwrap().flush_buf();
                    stack.last_mut().unwrap().inline_mode = InlineMode::Strikethrough;
                }
                Tag::Link { dest_url, .. } => {
                    stack.last_mut().unwrap().flush_buf();
                    stack.last_mut().unwrap().link_url = Some(dest_url.to_string());
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Heading(_) => {
                    let spans = stack.last_mut().unwrap().take_spans();
                    let level = match block_kind {
                        Some(BlockKind::Heading(l)) => l,
                        _ => 1,
                    };
                    blocks.push(MarkdownBlock::Heading(level, spans));
                    block_kind = None;
                }
                TagEnd::List(_) => {
                    list_depth = list_depth.saturating_sub(1);
                    // If we're back to depth 0 after lists, reset to paragraph mode
                    if list_depth == 0 {
                        block_kind = Some(BlockKind::Paragraph);
                    }
                }
                TagEnd::Item => {
                    let spans = stack.pop().unwrap_or_default().take_spans();
                    if !spans.is_empty() {
                        blocks.push(MarkdownBlock::ListItem(spans));
                    }
                    block_kind = Some(BlockKind::Paragraph);
                }
                TagEnd::Paragraph => {
                    // If inside a list item, paragraph end doesn't create a new block
                    if block_kind == Some(BlockKind::ListItem) {
                        // Stay in list item mode
                    } else {
                        flush_block(&mut blocks, &mut stack, &mut block_kind);
                        block_kind = Some(BlockKind::Paragraph);
                    }
                }
                TagEnd::BlockQuote(_) => {
                    let spans = stack.pop().unwrap_or_default().take_spans();
                    if !spans.is_empty() {
                        blocks.push(MarkdownBlock::Quote(spans));
                    }
                    block_kind = Some(BlockKind::Paragraph);
                }
                TagEnd::CodeBlock => {
                    let text = stack.pop().unwrap_or_default().take_text();
                    if !text.is_empty() {
                        blocks.push(MarkdownBlock::Code(text));
                    }
                    block_kind = Some(BlockKind::Paragraph);
                }
                TagEnd::Emphasis => {
                    stack.last_mut().unwrap().flush_buf();
                    stack.last_mut().unwrap().inline_mode = InlineMode::Plain;
                }
                TagEnd::Strong => {
                    stack.last_mut().unwrap().flush_buf();
                    stack.last_mut().unwrap().inline_mode = InlineMode::Plain;
                }
                TagEnd::Strikethrough => {
                    stack.last_mut().unwrap().flush_buf();
                    stack.last_mut().unwrap().inline_mode = InlineMode::Plain;
                }
                TagEnd::Link => {
                    // Link URL already consumed when building the span
                }
                _ => {}
            },
            Event::Text(text) => {
                stack.last_mut().unwrap().push_text(&text);
            }
            Event::Code(text) => {
                stack.last_mut().unwrap().flush_buf();
                stack
                    .last_mut()
                    .unwrap()
                    .spans
                    .push(InlineSpan::Code(text.to_string()));
            }
            Event::SoftBreak | Event::HardBreak => {
                if block_kind == Some(BlockKind::Code) {
                    stack.last_mut().unwrap().push_text("\n");
                } else {
                    stack.last_mut().unwrap().push_text(" ");
                }
            }
            Event::Rule => {
                flush_block(&mut blocks, &mut stack, &mut block_kind);
                blocks.push(MarkdownBlock::Rule);
                block_kind = Some(BlockKind::Paragraph);
            }
            _ => {}
        }
    }

    flush_block(&mut blocks, &mut stack, &mut block_kind);
    blocks
}

fn flush_block(
    blocks: &mut Vec<MarkdownBlock>,
    stack: &mut Vec<Accumulator>,
    block_kind: &mut Option<BlockKind>,
) {
    let acc = stack.last_mut().unwrap();
    let spans = acc.take_spans();
    if spans.is_empty() {
        return;
    }
    match block_kind {
        Some(BlockKind::Heading(level)) => {
            blocks.push(MarkdownBlock::Heading(*level, spans));
        }
        Some(BlockKind::Paragraph) | None => {
            blocks.push(MarkdownBlock::Paragraph(spans));
        }
        Some(BlockKind::Quote) => {
            blocks.push(MarkdownBlock::Quote(spans));
        }
        _ => {
            // ListItem and Code are handled in their End events
        }
    }
}

fn level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_markdown_extracts_heading_and_list_item() {
        let blocks = parse_markdown("# Relay\n\n- Terminal context");
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn parse_markdown_preserves_code_block_newlines() {
        let blocks = parse_markdown("```rust\nfn main() {}\nprintln!();\n```");
        assert!(
            matches!(&blocks[0], MarkdownBlock::Code(text) if text.contains('\n'))
        );
    }

    #[test]
    fn parse_markdown_handles_nested_lists() {
        let blocks = parse_markdown("- Item 1\n  - Nested item");
        // Should produce two list items (parent and nested)
        assert!(blocks.len() >= 2);
    }

    #[test]
    fn parse_markdown_handles_inline_bold() {
        let blocks = parse_markdown("**bold text** here");
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            MarkdownBlock::Paragraph(spans) => {
                assert!(spans.iter().any(|s| matches!(s, InlineSpan::Bold(_))));
            }
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn parse_markdown_handles_inline_italic() {
        let blocks = parse_markdown("*italic text* here");
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            MarkdownBlock::Paragraph(spans) => {
                assert!(spans.iter().any(|s| matches!(s, InlineSpan::Italic(_))));
            }
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn parse_markdown_handles_inline_strikethrough() {
        let blocks = parse_markdown("~~struck~~ text");
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            MarkdownBlock::Paragraph(spans) => {
                assert!(
                    spans
                        .iter()
                        .any(|s| matches!(s, InlineSpan::Strikethrough(_)))
                );
            }
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn parse_markdown_handles_inline_code() {
        let blocks = parse_markdown("use `println!` macro");
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            MarkdownBlock::Paragraph(spans) => {
                assert!(spans.iter().any(|s| matches!(s, InlineSpan::Code(_))));
            }
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn parse_markdown_handles_link() {
        let blocks = parse_markdown("[click here](https://example.com)");
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            MarkdownBlock::Paragraph(spans) => {
                assert!(spans.iter().any(|s| matches!(s, InlineSpan::Link { .. })));
            }
            _ => panic!("expected paragraph"),
        }
    }

    #[test]
    fn parse_markdown_empty_input_produces_no_blocks() {
        let blocks = parse_markdown("");
        assert!(blocks.is_empty());
    }
}
