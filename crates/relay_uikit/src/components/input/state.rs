use gpui::KeyDownEvent;
use unicode_segmentation::UnicodeSegmentation;

// ---------------------------------------------------------------------------
// Input value / action kinds
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputValueKind {
    Text,
    Number,
    Selection,
    Toggle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputActionKind {
    Changed(InputValueKind),
    CursorMoved,
    Submit,
    Cancel,
    Validate,
    Ignored,
}

impl InputActionKind {
    pub fn changes_value(self) -> bool {
        matches!(self, Self::Changed(_))
    }

    pub fn should_notify(self) -> bool {
        !matches!(self, Self::Ignored)
    }

    pub fn should_validate(self) -> bool {
        matches!(self, Self::Changed(_) | Self::Submit | Self::Validate)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationState {
    NotValidated,
    Valid,
    Invalid,
}

impl ValidationState {
    pub fn should_show_error(self) -> bool {
        matches!(self, Self::Invalid)
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
}

/// What a keystroke did to the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextInputAction {
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

    pub fn changes_text(self) -> bool {
        self.contract_kind().changes_value()
    }

    pub fn should_notify(self) -> bool {
        self.contract_kind().should_notify()
    }

    pub fn is_submit(self) -> bool {
        matches!(self, Self::Submit)
    }

    pub fn is_cancel(self) -> bool {
        matches!(self, Self::Cancel)
    }
}

impl TextInputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_text(text: impl Into<String>) -> Self {
        let value = text.into();
        let cursor = value.len();
        Self { value, cursor }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.value = text.into();
        self.cursor = self.value.len();
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

    pub fn handle_key(&mut self, event: &KeyDownEvent) -> TextInputAction {
        let keystroke = event.keystroke.clone().with_simulated_ime();
        let mods = keystroke.modifiers;

        match keystroke.key.as_str() {
            "enter" => TextInputAction::Submit,
            "escape" => TextInputAction::Cancel,
            "backspace" => {
                if self.delete_grapheme_before() {
                    TextInputAction::Changed
                } else {
                    TextInputAction::Ignored
                }
            }
            "delete" => {
                if self.delete_grapheme_after() {
                    TextInputAction::Changed
                } else {
                    TextInputAction::Ignored
                }
            }
            "left" => {
                if self.move_left() {
                    TextInputAction::CursorMoved
                } else {
                    TextInputAction::Ignored
                }
            }
            "right" => {
                if self.move_right() {
                    TextInputAction::CursorMoved
                } else {
                    TextInputAction::Ignored
                }
            }
            "home" => {
                if self.cursor != 0 {
                    self.cursor = 0;
                    TextInputAction::CursorMoved
                } else {
                    TextInputAction::Ignored
                }
            }
            "end" => {
                if self.cursor != self.value.len() {
                    self.cursor = self.value.len();
                    TextInputAction::CursorMoved
                } else {
                    TextInputAction::Ignored
                }
            }
            _ if !mods.control && !mods.alt && !mods.platform && !mods.function => {
                match keystroke
                    .key_char
                    .as_ref()
                    .filter(|text| text.chars().all(|c| !c.is_control()))
                {
                    Some(text) => {
                        self.insert(text);
                        TextInputAction::Changed
                    }
                    None => TextInputAction::Ignored,
                }
            }
            _ => TextInputAction::Ignored,
        }
    }

    pub fn handle_integer_key(
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
            _ if !mods.control && !mods.alt && !mods.platform && !mods.function => {
                match keystroke
                    .key_char
                    .as_ref()
                    .filter(|text| text.chars().all(|c| !c.is_control()))
                {
                    Some(text) if text.chars().all(|c| c.is_ascii_digit()) => {
                        self.insert(text);
                        TextInputAction::Changed
                    }
                    Some(text) if allow_negative && text == "-" && self.cursor == 0 => {
                        if self.value.starts_with('-') {
                            TextInputAction::Ignored
                        } else {
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

    pub fn handle_multiline_key(&mut self, event: &KeyDownEvent) -> TextInputAction {
        let keystroke = event.keystroke.clone().with_simulated_ime();
        let mods = keystroke.modifiers;

        match keystroke.key.as_str() {
            "enter" if mods.control || mods.platform => TextInputAction::Submit,
            "enter" => {
                self.insert("\n");
                TextInputAction::Changed
            }
            _ => self.handle_key(event),
        }
    }

    pub(crate) fn split(&self) -> (&str, &str) {
        let cursor = self.safe_cursor();
        self.value.split_at(cursor)
    }

    fn insert(&mut self, text: &str) {
        self.value.insert_str(self.cursor, text);
        self.cursor += text.len();
    }

    fn delete_grapheme_before(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        let prev = self.prev_boundary(self.cursor);
        self.value.replace_range(prev..self.cursor, "");
        self.cursor = prev;
        true
    }

    fn delete_grapheme_after(&mut self) -> bool {
        if self.cursor >= self.value.len() {
            return false;
        }
        let next = self.next_boundary(self.cursor);
        self.value.replace_range(self.cursor..next, "");
        true
    }

    fn move_left(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.cursor = self.prev_boundary(self.cursor);
        true
    }

    fn move_right(&mut self) -> bool {
        if self.cursor >= self.value.len() {
            return false;
        }
        self.cursor = self.next_boundary(self.cursor);
        true
    }

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
        assert!(TextInputAction::Changed.changes_text());
        assert!(!TextInputAction::CursorMoved.changes_text());
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
        let mut s = TextInputState::new();
        let mut k = key("a", Some("a"));
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
}
