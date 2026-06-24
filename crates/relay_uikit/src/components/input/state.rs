use std::ops::Range;

use gpui::{KeyDownEvent, UTF16Selection};
use unicode_segmentation::UnicodeSegmentation;

// ---------------------------------------------------------------------------
// Input value / action kinds
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputValueKind {
    Text,
    Number,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputActionKind {
    Changed(InputValueKind),
    CursorMoved,
    Submit,
    Cancel,
    Ignored,
}

impl InputActionKind {
    pub fn should_notify(self) -> bool {
        !matches!(self, Self::Ignored)
    }
}

// ---------------------------------------------------------------------------
// Text input state
// ---------------------------------------------------------------------------

/// The editable model for a [`crate::TextInput`].
#[derive(Debug, Clone, Default)]
pub struct TextInputState {
    value: String,
    cursor: usize,
    selection_anchor: Option<usize>,
    marked_range: Option<Range<usize>>,
}

/// What a keystroke did to the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextInputAction {
    Changed,
    CursorMoved,
    Submit,
    Cancel,
    Ignored,
}

impl TextInputAction {
    pub fn contract_kind(self) -> InputActionKind {
        self.contract_kind_for(InputValueKind::Text)
    }

    pub fn contract_kind_for(self, value_kind: InputValueKind) -> InputActionKind {
        match self {
            Self::Changed => InputActionKind::Changed(value_kind),
            Self::CursorMoved => InputActionKind::CursorMoved,
            Self::Submit => InputActionKind::Submit,
            Self::Cancel => InputActionKind::Cancel,
            Self::Ignored => InputActionKind::Ignored,
        }
    }

    pub fn should_notify(self) -> bool {
        self.contract_kind().should_notify()
    }
}

impl TextInputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_text(text: impl Into<String>) -> Self {
        let value = text.into();
        let cursor = value.len();
        Self {
            value,
            cursor,
            selection_anchor: None,
            marked_range: None,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_cursor(&mut self, byte: usize) {
        self.cursor = self.clamp_to_boundary(byte);
        self.selection_anchor = None;
        self.marked_range = None;
    }

    pub fn extend_selection_to(&mut self, byte: usize) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.safe_cursor());
        }
        self.cursor = self.clamp_to_boundary(byte);
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Returns the IME/composition range in UTF-8 byte offsets.
    pub fn marked_range(&self) -> Option<Range<usize>> {
        self.marked_range.clone()
    }

    // ------------------------------------------------------------------
    // Selection
    // ------------------------------------------------------------------

    /// Returns `Some((start_byte, end_byte))` when a non-empty selection exists.
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        let anchor = self.selection_anchor?;
        let start = anchor.min(self.cursor);
        let end = anchor.max(self.cursor);
        if start == end {
            None
        } else {
            Some((start, end))
        }
    }

    /// Returns the selected text slice, if any.
    pub fn selected_text(&self) -> Option<&str> {
        let (start, end) = self.selection_range()?;
        Some(&self.value[start..end])
    }

    /// Selects the entire value. Anchor goes to 0, cursor to the end.
    pub fn select_all(&mut self) {
        self.selection_anchor = Some(0);
        self.cursor = self.value.len();
    }

    /// Drops the selection without changing the text or cursor.
    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    /// Returns the current selection in UTF-16 code unit offsets.
    pub fn selected_text_range_utf16(&self) -> UTF16Selection {
        let range = self
            .selection_range()
            .map(|(start, end)| start..end)
            .unwrap_or_else(|| {
                let cursor = self.safe_cursor();
                cursor..cursor
            });

        UTF16Selection {
            range: self.range_to_utf16(&range),
            reversed: self
                .selection_anchor
                .is_some_and(|anchor| self.clamp_to_boundary(anchor) > self.safe_cursor()),
        }
    }

    /// Returns the current marked/composition range in UTF-16 code units.
    pub fn marked_text_range_utf16(&self) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    pub fn utf16_offset_for_byte(&self, byte: usize) -> usize {
        self.offset_to_utf16(byte)
    }

    pub fn byte_range_for_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.range_from_utf16(range_utf16)
    }

    /// Removes the selected text. Returns `true` if a non-empty selection
    /// was removed.
    pub fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.selection_range() {
            self.value.replace_range(start..end, "");
            self.cursor = start;
            self.selection_anchor = None;
            self.marked_range = None;
            true
        } else {
            false
        }
    }

    // ------------------------------------------------------------------
    // Clipboard helpers (to be called by the component / platform layer)
    // ------------------------------------------------------------------

    /// Returns a copy of the selected text, suitable for the system clipboard.
    pub fn copy_selection(&self) -> Option<String> {
        self.selected_text().map(|s| s.to_string())
    }

    /// Returns and removes the selected text, suitable for "cut".
    pub fn cut_selection(&mut self) -> Option<String> {
        let text = self.copy_selection();
        self.delete_selection();
        text
    }

    /// Inserts `text` at the cursor, replacing any active selection first.
    pub fn paste(&mut self, text: &str) {
        self.replace_text_in_range_utf16(None, text);
    }

    // ------------------------------------------------------------------
    // Mutators
    // ------------------------------------------------------------------

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.value = text.into();
        self.cursor = self.value.len();
        self.selection_anchor = None;
        self.marked_range = None;
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
        self.selection_anchor = None;
        self.marked_range = None;
    }

    /// Returns the substring for a platform-provided UTF-16 range, plus the
    /// actual clamped range if the request landed on non-boundary offsets.
    pub fn text_for_range_utf16(
        &self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        *adjusted_range = Some(self.range_to_utf16(&range));
        self.value.get(range).map(ToOwned::to_owned)
    }

    /// Replace a UTF-16 range, or the active marked/selected range when no
    /// explicit range is provided.
    pub fn replace_text_in_range_utf16(&mut self, range_utf16: Option<Range<usize>>, text: &str) {
        let range = self
            .resolve_replacement_range(range_utf16)
            .unwrap_or_else(|| {
                let cursor = self.safe_cursor();
                cursor..cursor
            });

        self.value.replace_range(range.clone(), text);
        self.cursor = range.start + text.len();
        self.selection_anchor = None;
        self.marked_range = None;
    }

    /// Replace text and update IME marked/selection ranges from UTF-16 input.
    pub fn replace_and_mark_text_in_range_utf16(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
    ) {
        let range = self
            .resolve_replacement_range(range_utf16)
            .unwrap_or_else(|| {
                let cursor = self.safe_cursor();
                cursor..cursor
            });

        self.value.replace_range(range.clone(), new_text);
        self.marked_range =
            (!new_text.is_empty()).then_some(range.start..range.start + new_text.len());

        if let Some(selected_range_utf16) = new_selected_range_utf16 {
            let selected_range = self.range_from_utf16_in_text(new_text, &selected_range_utf16);
            let start = range.start + selected_range.start;
            let end = range.start + selected_range.end;
            self.selection_anchor = Some(start);
            self.cursor = end;
        } else {
            self.selection_anchor = None;
            self.cursor = range.start + new_text.len();
        }
    }

    /// Clears the active IME/composition range while preserving the current
    /// text and selection.
    pub fn unmark_text(&mut self) {
        self.marked_range = None;
    }

    // ------------------------------------------------------------------
    // Key dispatch
    // ------------------------------------------------------------------

    pub(crate) fn handle_key(&mut self, event: &KeyDownEvent) -> TextInputAction {
        let keystroke = event.keystroke.clone().with_simulated_ime();
        let mods = keystroke.modifiers;

        match keystroke.key.as_str() {
            "enter" => TextInputAction::Submit,
            "escape" => TextInputAction::Cancel,

            "backspace" => {
                let changed = if mods.control {
                    self.delete_word_before()
                } else {
                    self.delete_selection() || self.delete_grapheme_before()
                };
                if changed {
                    TextInputAction::Changed
                } else {
                    TextInputAction::Ignored
                }
            }

            "delete" => {
                let changed = if mods.control {
                    self.delete_word_after()
                } else {
                    self.delete_selection() || self.delete_grapheme_after()
                };
                if changed {
                    TextInputAction::Changed
                } else {
                    TextInputAction::Ignored
                }
            }

            "left" => {
                if mods.control && mods.shift {
                    if self.selection_anchor.is_none() {
                        self.selection_anchor = Some(self.cursor);
                    }
                    if self.move_to_prev_word() {
                        TextInputAction::CursorMoved
                    } else {
                        TextInputAction::Ignored
                    }
                } else if mods.control {
                    self.clear_selection();
                    if self.move_to_prev_word() {
                        TextInputAction::CursorMoved
                    } else {
                        TextInputAction::Ignored
                    }
                } else if mods.shift {
                    if self.selection_anchor.is_none() {
                        self.selection_anchor = Some(self.cursor);
                    }
                    if self.move_left() {
                        TextInputAction::CursorMoved
                    } else {
                        TextInputAction::Ignored
                    }
                } else {
                    self.clear_selection();
                    if self.move_left() {
                        TextInputAction::CursorMoved
                    } else {
                        TextInputAction::Ignored
                    }
                }
            }

            "right" => {
                if mods.control && mods.shift {
                    if self.selection_anchor.is_none() {
                        self.selection_anchor = Some(self.cursor);
                    }
                    if self.move_to_next_word() {
                        TextInputAction::CursorMoved
                    } else {
                        TextInputAction::Ignored
                    }
                } else if mods.control {
                    self.clear_selection();
                    if self.move_to_next_word() {
                        TextInputAction::CursorMoved
                    } else {
                        TextInputAction::Ignored
                    }
                } else if mods.shift {
                    if self.selection_anchor.is_none() {
                        self.selection_anchor = Some(self.cursor);
                    }
                    if self.move_right() {
                        TextInputAction::CursorMoved
                    } else {
                        TextInputAction::Ignored
                    }
                } else {
                    self.clear_selection();
                    if self.move_right() {
                        TextInputAction::CursorMoved
                    } else {
                        TextInputAction::Ignored
                    }
                }
            }

            "home" => {
                if mods.shift {
                    if self.selection_anchor.is_none() {
                        self.selection_anchor = Some(self.cursor);
                    }
                    if self.cursor != 0 {
                        self.cursor = 0;
                        TextInputAction::CursorMoved
                    } else {
                        TextInputAction::Ignored
                    }
                } else {
                    self.clear_selection();
                    if self.cursor != 0 {
                        self.cursor = 0;
                        TextInputAction::CursorMoved
                    } else {
                        TextInputAction::Ignored
                    }
                }
            }

            "end" => {
                if mods.shift {
                    if self.selection_anchor.is_none() {
                        self.selection_anchor = Some(self.cursor);
                    }
                    if self.cursor != self.value.len() {
                        self.cursor = self.value.len();
                        TextInputAction::CursorMoved
                    } else {
                        TextInputAction::Ignored
                    }
                } else {
                    self.clear_selection();
                    if self.cursor != self.value.len() {
                        self.cursor = self.value.len();
                        TextInputAction::CursorMoved
                    } else {
                        TextInputAction::Ignored
                    }
                }
            }

            "a" if mods.control => {
                self.select_all();
                TextInputAction::CursorMoved
            }

            _ if !mods.control && !mods.alt && !mods.platform && !mods.function => {
                match keystroke
                    .key_char
                    .as_ref()
                    .filter(|text| text.chars().all(|c| !c.is_control()))
                {
                    Some(text) => {
                        self.delete_selection();
                        self.insert(text);
                        TextInputAction::Changed
                    }
                    None => TextInputAction::Ignored,
                }
            }
            _ => TextInputAction::Ignored,
        }
    }

    /// Handle only non-printable editing/navigation keys when the control is
    /// also wired into GPUI's platform text input pipeline.
    pub(crate) fn handle_platform_key(&mut self, event: &KeyDownEvent) -> TextInputAction {
        let keystroke = event.keystroke.clone().with_simulated_ime();
        let mods = keystroke.modifiers;

        match keystroke.key.as_str() {
            "enter" => TextInputAction::Submit,
            "escape" => TextInputAction::Cancel,
            "backspace" | "delete" | "left" | "right" | "home" | "end" => self.handle_key(event),
            "a" if mods.control => self.handle_key(event),
            _ => TextInputAction::Ignored,
        }
    }

    #[cfg(test)]
    pub(crate) fn handle_integer_key(
        &mut self,
        event: &KeyDownEvent,
        allow_negative: bool,
    ) -> TextInputAction {
        let keystroke = event.keystroke.clone().with_simulated_ime();
        let mods = keystroke.modifiers;

        match keystroke.key.as_str() {
            "enter" | "escape" | "backspace" | "delete" | "left" | "right" | "home" | "end" => {
                self.handle_key(event)
            }
            "a" if mods.control => self.handle_key(event),
            _ if !mods.control && !mods.alt && !mods.platform && !mods.function => {
                match keystroke
                    .key_char
                    .as_ref()
                    .filter(|text| text.chars().all(|c| !c.is_control()))
                {
                    Some(text) if text.chars().all(|c| c.is_ascii_digit()) => {
                        self.delete_selection();
                        self.insert(text);
                        TextInputAction::Changed
                    }
                    Some(text) if allow_negative && text == "-" && self.cursor == 0 => {
                        if self.value.starts_with('-') {
                            TextInputAction::Ignored
                        } else {
                            self.delete_selection();
                            self.insert(text);
                            TextInputAction::Changed
                        }
                    }
                    _ => TextInputAction::Ignored,
                }
            }
            _ => TextInputAction::Ignored,
        }
    }

    #[cfg(test)]
    pub(crate) fn handle_multiline_key(&mut self, event: &KeyDownEvent) -> TextInputAction {
        let keystroke = event.keystroke.clone().with_simulated_ime();
        let mods = keystroke.modifiers;

        match keystroke.key.as_str() {
            "enter" if mods.control || mods.platform => TextInputAction::Submit,
            "enter" => {
                self.delete_selection();
                self.insert("\n");
                TextInputAction::Changed
            }
            _ => self.handle_key(event),
        }
    }

    pub(crate) fn handle_platform_multiline_key(
        &mut self,
        event: &KeyDownEvent,
    ) -> TextInputAction {
        let keystroke = event.keystroke.clone().with_simulated_ime();
        let mods = keystroke.modifiers;

        match keystroke.key.as_str() {
            "enter" if mods.control || mods.platform => TextInputAction::Submit,
            "enter" => TextInputAction::Ignored,
            _ => self.handle_platform_key(event),
        }
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    #[cfg(test)]
    pub(crate) fn split(&self) -> (&str, &str) {
        let cursor = self.safe_cursor();
        self.value.split_at(cursor)
    }

    /// Insert text at cursor. Clears any selection anchor (the caller, i.e.
    /// `handle_key`, is expected to have already called `delete_selection()`
    /// if replacing active selection is desired).
    fn insert(&mut self, text: &str) {
        self.clear_selection();
        self.marked_range = None;
        self.value.insert_str(self.cursor, text);
        self.cursor += text.len();
    }

    fn delete_grapheme_before(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.clear_selection();
        let prev = self.prev_boundary(self.cursor);
        self.value.replace_range(prev..self.cursor, "");
        self.cursor = prev;
        self.marked_range = None;
        true
    }

    fn delete_grapheme_after(&mut self) -> bool {
        if self.cursor >= self.value.len() {
            return false;
        }
        self.clear_selection();
        let next = self.next_boundary(self.cursor);
        self.value.replace_range(self.cursor..next, "");
        self.marked_range = None;
        true
    }

    /// Move cursor one grapheme left (does *not* manage selection — the
    /// caller is responsible so that Shift variants keep the anchor).
    fn move_left(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.cursor = self.prev_boundary(self.cursor);
        true
    }

    /// Move cursor one grapheme right (does *not* manage selection — the
    /// caller is responsible so that Shift variants keep the anchor).
    fn move_right(&mut self) -> bool {
        if self.cursor >= self.value.len() {
            return false;
        }
        self.cursor = self.next_boundary(self.cursor);
        true
    }

    // ------------------------------------------------------------------
    // Word-level navigation
    // ------------------------------------------------------------------

    /// Move cursor to the start of the nearest non-whitespace word to the
    /// left, skipping any whitespace-only segments.
    fn move_to_prev_word(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        let target = self
            .value
            .split_word_bound_indices()
            .map(|(i, _)| i)
            .filter(|&i| i < self.cursor)
            .rev()
            .find(|&i| {
                !self.value[i..]
                    .chars()
                    .next()
                    .unwrap_or('\0')
                    .is_whitespace()
            })
            .unwrap_or(0);
        if target != self.cursor {
            self.cursor = target;
            true
        } else {
            false
        }
    }

    /// Move cursor to the end of the nearest non-whitespace word to the
    /// right, skipping any whitespace-only segments.
    fn move_to_next_word(&mut self) -> bool {
        if self.cursor >= self.value.len() {
            return false;
        }
        let boundaries: Vec<usize> = self
            .value
            .split_word_bound_indices()
            .map(|(i, _)| i)
            .collect();
        // Walk forward to find the next boundary that ends a non-whitespace
        // segment (i.e. the segment between the previous boundary and this
        // one contains something other than just whitespace).
        let target = boundaries
            .iter()
            .filter(|&&b| b > self.cursor)
            .find(|&&b| {
                let prev = boundaries
                    .iter()
                    .rev()
                    .find(|&&pb| pb < b)
                    .copied()
                    .unwrap_or(0);
                !self.value[prev..b].chars().all(|c| c.is_whitespace())
            })
            .copied()
            .unwrap_or(self.value.len());
        if target != self.cursor {
            self.cursor = target;
            true
        } else {
            false
        }
    }

    fn delete_word_before(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.clear_selection();
        let start = self
            .value
            .split_word_bound_indices()
            .map(|(i, _)| i)
            .filter(|&i| i < self.cursor)
            .rev()
            .find(|&i| {
                !self.value[i..]
                    .chars()
                    .next()
                    .unwrap_or('\0')
                    .is_whitespace()
            })
            .unwrap_or(0);
        self.value.replace_range(start..self.cursor, "");
        self.cursor = start;
        self.marked_range = None;
        true
    }

    fn delete_word_after(&mut self) -> bool {
        if self.cursor >= self.value.len() {
            return false;
        }
        self.clear_selection();
        let boundaries: Vec<usize> = self
            .value
            .split_word_bound_indices()
            .map(|(i, _)| i)
            .collect();
        let end = boundaries
            .iter()
            .filter(|&&b| b > self.cursor)
            .find(|&&b| {
                let prev = boundaries
                    .iter()
                    .rev()
                    .find(|&&pb| pb < b)
                    .copied()
                    .unwrap_or(0);
                !self.value[prev..b].chars().all(|c| c.is_whitespace())
            })
            .copied()
            .unwrap_or(self.value.len());
        self.value.replace_range(self.cursor..end, "");
        self.marked_range = None;
        true
    }

    fn resolve_replacement_range(&self, range_utf16: Option<Range<usize>>) -> Option<Range<usize>> {
        range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or_else(|| self.marked_range.clone())
            .or_else(|| self.selection_range().map(|(start, end)| start..end))
    }

    // ------------------------------------------------------------------
    // Grapheme boundary helpers
    // ------------------------------------------------------------------

    fn prev_boundary(&self, byte: usize) -> usize {
        let byte = self.clamp_to_boundary(byte);
        self.value[..byte]
            .grapheme_indices(true)
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn next_boundary(&self, byte: usize) -> usize {
        let byte = self.clamp_to_boundary(byte);
        self.value[byte..]
            .grapheme_indices(true)
            .nth(1)
            .map(|(i, _)| byte + i)
            .unwrap_or(self.value.len())
    }

    /// Clamp `cursor` to a valid UTF-8 character boundary within `[0, len]`.
    ///
    /// This is a defensive guard: if `cursor` was set to an invalid position
    /// (e.g. via `update_silent` or external mutation), `split_at` / string
    /// slicing would panic. This method walks back to the nearest boundary.
    fn safe_cursor(&self) -> usize {
        self.clamp_to_boundary(self.cursor)
    }

    fn clamp_to_boundary(&self, mut byte: usize) -> usize {
        let len = self.value.len();
        if byte > len {
            byte = len;
        }
        while byte > 0 && !self.value.is_char_boundary(byte) {
            byte -= 1;
        }
        byte
    }

    fn offset_from_utf16_in_text(&self, text: &str, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for ch in text.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }

        utf8_offset.min(text.len())
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        self.offset_from_utf16_in_text(&self.value, offset)
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        let offset = self.clamp_to_boundary(offset);

        for ch in self.value.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }

        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn range_from_utf16_in_text(&self, text: &str, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16_in_text(text, range_utf16.start)
            ..self.offset_from_utf16_in_text(text, range_utf16.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(name: &str, ch: Option<&str>) -> KeyDownEvent {
        KeyDownEvent {
            keystroke: gpui::Keystroke {
                key: name.to_string(),
                key_char: ch.map(|c| c.to_string()),
                ..Default::default()
            },
            is_held: false,
            prefer_character_input: false,
        }
    }

    fn key_with_mods(name: &str, ch: Option<&str>, control: bool, shift: bool) -> KeyDownEvent {
        KeyDownEvent {
            keystroke: gpui::Keystroke {
                key: name.to_string(),
                key_char: ch.map(|c| c.to_string()),
                modifiers: gpui::Modifiers {
                    control,
                    shift,
                    ..Default::default()
                },
            },
            is_held: false,
            prefer_character_input: false,
        }
    }

    // ------------------------------------------------------------------
    // Existing tests (preserved)
    // ------------------------------------------------------------------

    #[test]
    fn typing_inserts_at_cursor() {
        let mut s = TextInputState::new();
        assert_eq!(s.handle_key(&key("h", Some("h"))), TextInputAction::Changed);
        s.handle_key(&key("i", Some("i")));
        assert_eq!(s.value(), "hi");
    }

    #[test]
    fn backspace_removes_grapheme_before_cursor() {
        let mut s = TextInputState::with_text("hi");
        assert_eq!(
            s.handle_key(&key("backspace", None)),
            TextInputAction::Changed
        );
        assert_eq!(s.value(), "h");
    }

    #[test]
    fn arrows_move_cursor_and_insert_lands_mid_string() {
        let mut s = TextInputState::with_text("ac");
        assert_eq!(
            s.handle_key(&key("left", None)),
            TextInputAction::CursorMoved
        );
        s.handle_key(&key("b", Some("b")));
        assert_eq!(s.value(), "abc");
    }

    #[test]
    fn action_helpers_distinguish_text_changes_from_cursor_motion() {
        assert_eq!(
            TextInputAction::Changed.contract_kind(),
            InputActionKind::Changed(InputValueKind::Text)
        );
        assert_eq!(
            TextInputAction::CursorMoved.contract_kind(),
            InputActionKind::CursorMoved
        );
        assert!(TextInputAction::CursorMoved.should_notify());
        assert!(!TextInputAction::Ignored.should_notify());
    }

    #[test]
    fn text_input_action_maps_to_contract_action_kind() {
        assert_eq!(
            TextInputAction::Changed.contract_kind(),
            InputActionKind::Changed(InputValueKind::Text)
        );
    }

    #[test]
    fn input_action_can_map_to_numeric_contract_kind() {
        assert_eq!(
            TextInputAction::Changed.contract_kind_for(InputValueKind::Number),
            InputActionKind::Changed(InputValueKind::Number)
        );
    }

    #[test]
    fn enter_and_escape_report_intents() {
        let mut s = TextInputState::with_text("x");
        assert_eq!(s.handle_key(&key("enter", None)), TextInputAction::Submit);
        assert_eq!(s.handle_key(&key("escape", None)), TextInputAction::Cancel);
    }

    #[test]
    fn platform_key_mode_ignores_printable_text() {
        let mut s = TextInputState::with_text("x");
        assert_eq!(
            s.handle_platform_key(&key("a", Some("a"))),
            TextInputAction::Ignored
        );
        assert_eq!(s.value(), "x");
    }

    #[test]
    fn extend_selection_to_anchors_existing_cursor() {
        let mut s = TextInputState::with_text("hello");
        s.set_cursor(2);
        s.extend_selection_to(4);
        assert_eq!(s.selection_range(), Some((2, 4)));
    }

    #[test]
    fn multiline_enter_inserts_newline() {
        let mut s = TextInputState::with_text("a");

        assert_eq!(
            s.handle_multiline_key(&key("enter", None)),
            TextInputAction::Changed
        );
        assert_eq!(s.value(), "a\n");
    }

    #[test]
    fn multiline_control_enter_reports_submit() {
        let mut s = TextInputState::with_text("a");
        let mut event = key("enter", None);
        event.keystroke.modifiers.control = true;

        assert_eq!(s.handle_multiline_key(&event), TextInputAction::Submit);
        assert_eq!(s.value(), "a");
    }

    #[test]
    fn home_end_jump_to_edges() {
        let mut s = TextInputState::with_text("abc");
        assert_eq!(
            s.handle_key(&key("home", None)),
            TextInputAction::CursorMoved
        );
        assert_eq!(s.cursor(), 0);
        assert_eq!(
            s.handle_key(&key("end", None)),
            TextInputAction::CursorMoved
        );
        assert_eq!(s.cursor(), 3);
    }

    #[test]
    fn cjk_input_via_key_char() {
        let mut s = TextInputState::new();
        s.handle_key(&key("中", Some("中")));
        s.handle_key(&key("文", Some("文")));
        assert_eq!(s.value(), "中文");
        s.handle_key(&key("backspace", None));
        assert_eq!(s.value(), "中");
    }

    #[test]
    fn ctrl_combo_is_ignored() {
        // Ctrl+A is special-cased (select-all); other ctrl combos are ignored.
        let mut s = TextInputState::new();
        let mut k = key("b", Some("b"));
        k.keystroke.modifiers.control = true;
        assert_eq!(s.handle_key(&k), TextInputAction::Ignored);
        assert!(s.is_empty());
    }

    #[test]
    fn integer_input_accepts_digits() {
        let mut s = TextInputState::new();

        assert_eq!(
            s.handle_integer_key(&key("1", Some("1")), false),
            TextInputAction::Changed
        );
        assert_eq!(s.value(), "1");
    }

    #[test]
    fn integer_input_rejects_letters() {
        let mut s = TextInputState::new();

        assert_eq!(
            s.handle_integer_key(&key("a", Some("a")), false),
            TextInputAction::Ignored
        );
        assert!(s.is_empty());
    }

    #[test]
    fn integer_input_accepts_single_leading_minus_when_configured() {
        let mut s = TextInputState::new();

        assert_eq!(
            s.handle_integer_key(&key("-", Some("-")), true),
            TextInputAction::Changed
        );
        assert_eq!(
            s.handle_integer_key(&key("-", Some("-")), true),
            TextInputAction::Ignored
        );
        assert_eq!(s.value(), "-");
    }

    #[test]
    fn split_does_not_panic_on_invalid_cursor() {
        let mut s = TextInputState::with_text("héllo"); // é is 2 bytes
        // Set cursor to a non-char-boundary position (byte 2, inside é).
        s.cursor = 2;
        // split() should clamp to the nearest boundary, not panic.
        let (before, after) = s.split();
        assert_eq!(before, "h");
        assert_eq!(after, "éllo");
    }

    #[test]
    fn split_clamps_cursor_beyond_length() {
        let s = TextInputState::with_text("abc");
        let mut s = s;
        s.cursor = 100;
        let (before, after) = s.split();
        assert_eq!(before, "abc");
        assert_eq!(after, "");
    }

    // ------------------------------------------------------------------
    // New tests: selection
    // ------------------------------------------------------------------

    #[test]
    fn basic_selection_with_shift_right() {
        let mut s = TextInputState::with_text("hello");
        // Move cursor left twice to position 3 ("hel|lo")
        s.handle_key(&key("left", None));
        s.handle_key(&key("left", None));
        assert_eq!(s.cursor(), 3);

        // Shift+Right: begin selection, cursor moves right
        let ev = key_with_mods("right", None, false, true);
        assert_eq!(s.handle_key(&ev), TextInputAction::CursorMoved);
        assert_eq!(s.cursor(), 4);
        assert_eq!(s.selection_range(), Some((3, 4)));
        assert_eq!(s.selected_text(), Some("l"));
    }

    #[test]
    fn selection_anchor_persists_across_multiple_shift_moves() {
        let mut s = TextInputState::with_text("hello");
        // Set cursor to 1 ("h|ello")
        s.handle_key(&key("home", None));
        s.handle_key(&key("right", None));
        assert_eq!(s.cursor(), 1);

        // Shift+Right three times: anchor stays at 1
        for _ in 0..3 {
            s.handle_key(&key_with_mods("right", None, false, true));
        }
        assert_eq!(s.cursor(), 4);
        assert_eq!(s.selection_range(), Some((1, 4)));
        assert_eq!(s.selected_text(), Some("ell"));
    }

    #[test]
    fn select_all_via_ctrl_a() {
        let mut s = TextInputState::with_text("hello world");
        let mut k = key("a", Some("a"));
        k.keystroke.modifiers.control = true;

        assert_eq!(s.handle_key(&k), TextInputAction::CursorMoved);
        assert_eq!(s.selection_range(), Some((0, 11)));
        assert_eq!(s.selected_text(), Some("hello world"));
    }

    #[test]
    fn shift_home_selects_to_beginning() {
        let mut s = TextInputState::with_text("hello");
        s.handle_key(&key("end", None)); // cursor at 5
        s.handle_key(&key("left", None)); // cursor at 4
        s.handle_key(&key("left", None)); // cursor at 3

        let ev = key_with_mods("home", None, false, true);
        assert_eq!(s.handle_key(&ev), TextInputAction::CursorMoved);
        assert_eq!(s.cursor(), 0);
        assert_eq!(s.selection_range(), Some((0, 3)));
    }

    #[test]
    fn shift_end_selects_to_end() {
        let mut s = TextInputState::with_text("hello");
        s.handle_key(&key("home", None)); // cursor at 0
        s.handle_key(&key("right", None)); // cursor at 1

        let ev = key_with_mods("end", None, false, true);
        assert_eq!(s.handle_key(&ev), TextInputAction::CursorMoved);
        assert_eq!(s.cursor(), 5);
        assert_eq!(s.selection_range(), Some((1, 5)));
    }

    #[test]
    fn selection_cleared_on_typing() {
        let mut s = TextInputState::with_text("hello");
        s.select_all();
        assert!(s.selection_range().is_some());

        // Typing a character replaces the selection and clears the anchor
        s.handle_key(&key("x", Some("x")));
        assert_eq!(s.value(), "x");
        assert!(s.selection_range().is_none());
    }

    #[test]
    fn selection_cleared_on_cursor_movement_without_shift() {
        let mut s = TextInputState::with_text("hello");
        s.select_all();
        assert!(s.selection_range().is_some());

        // Plain left arrow should clear selection
        s.handle_key(&key("left", None));
        assert!(s.selection_range().is_none());
    }

    #[test]
    fn delete_selection_removes_text() {
        let mut s = TextInputState::with_text("hello world");
        // Select "ello "
        s.selection_anchor = Some(1);
        s.cursor = 6;

        assert!(s.delete_selection());
        assert_eq!(s.value(), "hworld");
        assert_eq!(s.cursor(), 1);
        assert!(s.selection_range().is_none());
    }

    #[test]
    fn delete_selection_returns_false_when_nothing_selected() {
        let mut s = TextInputState::with_text("hello");
        assert!(!s.delete_selection());
        assert_eq!(s.value(), "hello");
    }

    #[test]
    fn clear_selection_drops_anchor_but_keeps_text_and_cursor() {
        let mut s = TextInputState::with_text("hello");
        s.selection_anchor = Some(2);
        s.cursor = 4;
        s.clear_selection();
        assert_eq!(s.selection_range(), None);
        assert_eq!(s.value(), "hello");
        assert_eq!(s.cursor(), 4);
    }

    // ------------------------------------------------------------------
    // New tests: word navigation
    // ------------------------------------------------------------------

    #[test]
    fn ctrl_left_moves_to_prev_word() {
        let mut s = TextInputState::with_text("hello world");
        // cursor at end (11)
        let ev = key_with_mods("left", None, true, false);
        s.handle_key(&ev);
        assert_eq!(s.cursor(), 6); // start of "world"
        s.handle_key(&ev);
        assert_eq!(s.cursor(), 0); // start of "hello" (skipping the space)
        s.handle_key(&ev); // already at 0, should be ignored
    }

    #[test]
    fn ctrl_right_moves_to_next_word() {
        let mut s = TextInputState::with_text("hello world");
        s.handle_key(&key("home", None)); // cursor at 0
        let ev = key_with_mods("right", None, true, false);
        s.handle_key(&ev);
        assert_eq!(s.cursor(), 5); // after "hello"
        s.handle_key(&ev);
        assert_eq!(s.cursor(), 11); // end of "world" (skipping the space)
    }

    #[test]
    fn ctrl_shift_left_selects_by_word() {
        let mut s = TextInputState::with_text("hello world");
        // cursor at end (11)
        let ev = key_with_mods("left", None, true, true);
        s.handle_key(&ev);
        assert_eq!(s.cursor(), 6);
        assert_eq!(s.selection_range(), Some((6, 11)));
        assert_eq!(s.selected_text(), Some("world"));
    }

    #[test]
    fn ctrl_shift_right_selects_by_word() {
        let mut s = TextInputState::with_text("hello world");
        s.handle_key(&key("home", None)); // cursor at 0
        let ev = key_with_mods("right", None, true, true);
        s.handle_key(&ev);
        assert_eq!(s.cursor(), 5);
        assert_eq!(s.selection_range(), Some((0, 5)));
        assert_eq!(s.selected_text(), Some("hello"));
    }

    // ------------------------------------------------------------------
    // New tests: word deletion
    // ------------------------------------------------------------------

    #[test]
    fn ctrl_backspace_deletes_prev_word() {
        let mut s = TextInputState::with_text("hello world");
        let mut ev = key("backspace", None);
        ev.keystroke.modifiers.control = true;
        assert_eq!(s.handle_key(&ev), TextInputAction::Changed);
        assert_eq!(s.value(), "hello ");
        assert_eq!(s.cursor(), 6);
    }

    #[test]
    fn ctrl_delete_deletes_next_word() {
        let mut s = TextInputState::with_text("hello world");
        s.handle_key(&key("home", None)); // cursor at 0
        let mut ev = key("delete", None);
        ev.keystroke.modifiers.control = true;
        assert_eq!(s.handle_key(&ev), TextInputAction::Changed);
        assert_eq!(s.value(), " world");
        assert_eq!(s.cursor(), 0);
    }

    // ------------------------------------------------------------------
    // New tests: clipboard
    // ------------------------------------------------------------------

    #[test]
    fn copy_selection_returns_selected_text() {
        let mut s = TextInputState::with_text("hello world");
        s.selection_anchor = Some(0);
        s.cursor = 5;
        assert_eq!(s.copy_selection(), Some("hello".to_string()));
    }

    #[test]
    fn copy_selection_returns_none_when_nothing_selected() {
        let s = TextInputState::with_text("hello");
        assert_eq!(s.copy_selection(), None);
    }

    #[test]
    fn cut_selection_returns_and_removes_text() {
        let mut s = TextInputState::with_text("hello world");
        s.selection_anchor = Some(0);
        s.cursor = 5;
        let cut = s.cut_selection();
        assert_eq!(cut, Some("hello".to_string()));
        assert_eq!(s.value(), " world");
        assert_eq!(s.cursor(), 0);
    }

    #[test]
    fn paste_inserts_at_cursor() {
        let mut s = TextInputState::with_text("hello");
        s.paste(" world");
        assert_eq!(s.value(), "hello world");
        assert_eq!(s.cursor(), 11);
    }

    #[test]
    fn paste_replaces_selection() {
        let mut s = TextInputState::with_text("hello world");
        s.selection_anchor = Some(0);
        s.cursor = 5; // "hello" selected
        s.paste("hi");
        assert_eq!(s.value(), "hi world");
        assert_eq!(s.cursor(), 2);
        assert!(s.selection_range().is_none());
    }

    // ------------------------------------------------------------------
    // New tests: edge cases
    // ------------------------------------------------------------------

    #[test]
    fn selection_range_returns_none_for_empty_selection() {
        let mut s = TextInputState::with_text("hello");
        s.selection_anchor = Some(3);
        s.cursor = 3; // anchor == cursor: empty selection
        assert_eq!(s.selection_range(), None);
        assert_eq!(s.selected_text(), None);
    }

    #[test]
    fn selection_range_works_regardless_of_anchor_cursor_order() {
        let mut s = TextInputState::with_text("hello");
        // anchor after cursor
        s.selection_anchor = Some(4);
        s.cursor = 1;
        assert_eq!(s.selection_range(), Some((1, 4)));

        // cursor after anchor
        s.selection_anchor = Some(1);
        s.cursor = 4;
        assert_eq!(s.selection_range(), Some((1, 4)));
    }

    #[test]
    fn select_all_on_empty_input() {
        let mut s = TextInputState::new();
        s.select_all();
        assert_eq!(s.cursor(), 0);
        assert_eq!(s.selection_range(), None); // empty selection
    }

    #[test]
    fn set_text_clears_selection() {
        let mut s = TextInputState::with_text("hello");
        s.select_all();
        assert!(s.selection_range().is_some());
        s.set_text("world");
        assert_eq!(s.value(), "world");
        assert_eq!(s.cursor(), 5);
        assert!(s.selection_range().is_none());
    }

    #[test]
    fn clear_clears_selection() {
        let mut s = TextInputState::with_text("hello");
        s.select_all();
        assert!(s.selection_range().is_some());
        s.clear();
        assert!(s.is_empty());
        assert_eq!(s.cursor(), 0);
        assert!(s.selection_range().is_none());
    }

    #[test]
    fn backspace_deletes_selection_when_present() {
        let mut s = TextInputState::with_text("hello world");
        s.selection_anchor = Some(6);
        s.cursor = 11; // "world" selected
        assert_eq!(
            s.handle_key(&key("backspace", None)),
            TextInputAction::Changed
        );
        assert_eq!(s.value(), "hello ");
        assert_eq!(s.cursor(), 6);
    }

    #[test]
    fn delete_deletes_selection_when_present() {
        let mut s = TextInputState::with_text("hello world");
        s.selection_anchor = Some(0);
        s.cursor = 6; // "hello " selected
        assert_eq!(s.handle_key(&key("delete", None)), TextInputAction::Changed);
        assert_eq!(s.value(), "world");
        assert_eq!(s.cursor(), 0);
    }

    #[test]
    fn shift_left_starts_selection() {
        let mut s = TextInputState::with_text("hello");
        // cursor at end
        let ev = key_with_mods("left", None, false, true);
        s.handle_key(&ev);
        assert_eq!(s.cursor(), 4);
        assert_eq!(s.selection_range(), Some((4, 5)));
    }

    #[test]
    fn shift_right_then_shift_left_narrows_selection() {
        let mut s = TextInputState::with_text("hello");
        s.handle_key(&key("home", None)); // cursor at 0
        let right_shift = key_with_mods("right", None, false, true);
        let left_shift = key_with_mods("left", None, false, true);

        s.handle_key(&right_shift);
        s.handle_key(&right_shift);
        s.handle_key(&right_shift);
        assert_eq!(s.cursor(), 3);
        assert_eq!(s.selection_range(), Some((0, 3)));

        // Shift+Left shrinks selection
        s.handle_key(&left_shift);
        assert_eq!(s.cursor(), 2);
        assert_eq!(s.selection_range(), Some((0, 2)));
    }

    #[test]
    fn move_to_prev_word_at_start_returns_false() {
        let mut s = TextInputState::new();
        // empty, cursor at 0
        assert!(!s.move_to_prev_word());
    }

    #[test]
    fn move_to_next_word_at_end_returns_false() {
        let mut s = TextInputState::with_text("hi");
        // cursor at end
        assert!(!s.move_to_next_word());
    }

    #[test]
    fn word_navigation_handles_punctuation() {
        let mut s = TextInputState::with_text("hello, world!");
        s.handle_key(&key("home", None)); // cursor at 0
        let ctrl_right = key_with_mods("right", None, true, false);

        s.handle_key(&ctrl_right);
        assert_eq!(s.cursor(), 5); // after "hello"
        s.handle_key(&ctrl_right);
        assert_eq!(s.cursor(), 6); // after ","
        // Whitespace segment is skipped, so we jump to the end of "world"
        s.handle_key(&ctrl_right);
        assert_eq!(s.cursor(), 12); // after "world" (skipped space)
        s.handle_key(&ctrl_right);
        assert_eq!(s.cursor(), 13); // after "!" (end of string)
    }

    #[test]
    fn word_deletion_clears_selection() {
        let mut s = TextInputState::with_text("hello world");
        s.select_all();
        assert!(s.selection_range().is_some());

        // Ctrl+Backspace: word deletion should clear selection
        let mut ev = key("backspace", None);
        ev.keystroke.modifiers.control = true;
        s.handle_key(&ev);
        assert!(s.selection_range().is_none());
    }

    #[test]
    fn copy_selection_returns_none_for_empty_value() {
        let s = TextInputState::new();
        assert_eq!(s.copy_selection(), None);
    }

    #[test]
    fn cut_selection_returns_none_when_nothing_selected() {
        let mut s = TextInputState::with_text("hello");
        assert_eq!(s.cut_selection(), None);
        assert_eq!(s.value(), "hello");
    }

    #[test]
    fn integer_ctrl_a_selects_all() {
        let mut s = TextInputState::with_text("42");
        let mut k = key("a", Some("a"));
        k.keystroke.modifiers.control = true;
        assert_eq!(
            s.handle_integer_key(&k, false),
            TextInputAction::CursorMoved
        );
        assert_eq!(s.selection_range(), Some((0, 2)));
    }

    #[test]
    fn multiline_enter_replaces_selection() {
        let mut s = TextInputState::with_text("hello world");
        s.select_all();
        assert_eq!(
            s.handle_multiline_key(&key("enter", None)),
            TextInputAction::Changed
        );
        assert_eq!(s.value(), "\n");
        assert!(s.selection_range().is_none());
    }

    #[test]
    fn selected_text_range_utf16_tracks_reversed_selection() {
        let mut s = TextInputState::with_text("a😀b");
        s.selection_anchor = Some("a😀".len());
        s.cursor = "a".len();

        let selection = s.selected_text_range_utf16();
        assert_eq!(selection.range, 1..3);
        assert!(selection.reversed);
    }

    #[test]
    fn text_for_range_utf16_adjusts_to_unicode_boundaries() {
        let s = TextInputState::with_text("a😀b");
        let mut adjusted = None;

        let text = s.text_for_range_utf16(1..3, &mut adjusted);

        assert_eq!(text.as_deref(), Some("😀"));
        assert_eq!(adjusted, Some(1..3));
    }

    #[test]
    fn replace_text_in_range_utf16_replaces_active_selection() {
        let mut s = TextInputState::with_text("a😀b");
        s.selection_anchor = Some("a".len());
        s.cursor = "a😀".len();

        s.replace_text_in_range_utf16(None, "x");

        assert_eq!(s.value(), "axb");
        assert_eq!(s.cursor(), 2);
        assert!(s.selection_range().is_none());
    }

    #[test]
    fn replace_and_mark_text_in_range_utf16_sets_marked_and_selected_ranges() {
        let mut s = TextInputState::new();

        s.replace_and_mark_text_in_range_utf16(None, "你好", Some(1..2));

        assert_eq!(s.value(), "你好");
        assert_eq!(s.marked_range(), Some(0.."你好".len()));
        assert_eq!(s.selected_text_range_utf16().range, 1..2);
    }

    #[test]
    fn unmark_text_clears_marked_range_without_touching_text() {
        let mut s = TextInputState::new();
        s.replace_and_mark_text_in_range_utf16(None, "relay", Some(0..5));

        s.unmark_text();

        assert_eq!(s.value(), "relay");
        assert_eq!(s.marked_range(), None);
    }
}
