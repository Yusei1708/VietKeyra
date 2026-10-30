//! VietKeyra Core Telex Engine
//!
//! Pure Rust, zero-dependency, sub-microsecond latency Vietnamese Telex IME engine.
//! Provides O(1) zero-allocation Unicode NFC tables, phonotactic syllable parsing, Modern & Traditional
//! tone placement, repeat-key reversion state machine, and exact UTF-8 byte calculations.

pub mod nfc;
pub mod syllable;
pub mod telex;
pub mod tone;
pub mod undo;

pub use tone::TonePlacementStyle;
pub use undo::{StepTransform, UndoHistory, WordStep};

/// Input mode: Vietnamese Telex or English pass-through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Vietnamese,
    English,
}

impl InputMode {
    /// Returns `true` if current mode is Vietnamese.
    #[inline]
    pub fn is_vietnamese(&self) -> bool {
        matches!(self, Self::Vietnamese)
    }

    /// Returns `true` if current mode is English.
    #[inline]
    pub fn is_english(&self) -> bool {
        matches!(self, Self::English)
    }
}

/// Action to be executed by the Wayland input method daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAction {
    /// Pass raw key through to Wayland compositor as-is.
    PassThrough,
    /// Replace previous word or characters in surrounding text.
    Replace {
        delete_chars: usize,
        delete_bytes: usize, // Required for Wayland delete_surrounding_text
        text: String,
    },
    /// Direct commit (space, punctuation, enter).
    Commit { text: String },
}

/// Main Vietnamese Telex IME state machine engine.
pub struct VietKeyraEngine {
    mode: InputMode,
    style: TonePlacementStyle,
    current_word: String,
    history: UndoHistory,
}

impl Default for VietKeyraEngine {
    fn default() -> Self {
        Self::new(TonePlacementStyle::default())
    }
}

impl VietKeyraEngine {
    /// Creates a new `VietKeyraEngine` with the given tone placement style.
    pub fn new(style: TonePlacementStyle) -> Self {
        Self {
            mode: InputMode::Vietnamese,
            style,
            current_word: String::with_capacity(32),
            history: UndoHistory::new(),
        }
    }

    /// Sets the input mode (Vietnamese or English).
    pub fn set_mode(&mut self, mode: InputMode) {
        if self.mode != mode {
            self.reset_word();
            self.mode = mode;
        }
    }

    /// Returns the active input mode.
    #[inline]
    pub fn get_mode(&self) -> InputMode {
        self.mode
    }

    /// Sets the tone placement style (Modern or Traditional).
    pub fn set_style(&mut self, style: TonePlacementStyle) {
        self.style = style;
    }

    /// Returns the active tone placement style.
    #[inline]
    pub fn style(&self) -> TonePlacementStyle {
        self.style
    }

    /// Returns the current word buffer.
    #[inline]
    pub fn current_word(&self) -> &str {
        &self.current_word
    }

    /// Resets the current word buffer and undo history.
    pub fn reset_word(&mut self) {
        self.current_word.clear();
        self.history.clear();
    }

    /// Returns `true` if the active word buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.current_word.is_empty()
    }

    /// Returns the number of recorded undo steps in the active word.
    #[inline]
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Toggles the input mode between Vietnamese and English.
    pub fn toggle_mode(&mut self) -> InputMode {
        let new_mode = match self.mode {
            InputMode::Vietnamese => InputMode::English,
            InputMode::English => InputMode::Vietnamese,
        };
        self.set_mode(new_mode);
        new_mode
    }

    /// Handles a Backspace key event.
    ///
    /// In Vietnamese mode, pops the last character from `current_word` and returns
    /// `KeyAction::Replace` with exact UTF-8 `delete_bytes` for Wayland compositor deletion.
    /// If the word buffer is or becomes empty, the undo history is cleanly cleared to prevent
    /// stale transformation states from affecting subsequent typing.
    /// In English mode or when the word buffer is empty, returns `KeyAction::PassThrough`.
    pub fn handle_backspace(&mut self) -> KeyAction {
        if self.mode == InputMode::English {
            self.reset_word();
            return KeyAction::PassThrough;
        }

        if self.current_word.is_empty() {
            self.history.clear();
            return KeyAction::PassThrough;
        }

        self.history.pop();
        if let Some(last_char) = self.current_word.pop() {
            if self.current_word.is_empty() {
                self.history.clear();
            }
            KeyAction::Replace {
                delete_chars: 1,
                delete_bytes: last_char.len_utf8(),
                text: String::new(),
            }
        } else {
            self.history.clear();
            KeyAction::PassThrough
        }
    }

    /// Processes an incoming character according to Telex orthography rules.
    pub fn process_char(&mut self, ch: char) -> KeyAction {
        if self.mode == InputMode::English {
            self.reset_word();
            return KeyAction::PassThrough;
        }

        // Non-alphabetic characters (space, punctuation, numbers, symbols) reset the word buffer
        if !ch.is_alphabetic() {
            self.reset_word();
            return KeyAction::Commit {
                text: ch.to_string(),
            };
        }

        let last_step = self.history.last().cloned();
        let res = telex::process_telex_key(&self.current_word, ch, self.style, last_step.as_ref());

        self.current_word = res.new_word.clone();
        self.history.push(WordStep {
            key: ch,
            word_after: res.new_word,
            transform: res.transform,
        });

        KeyAction::Replace {
            delete_chars: res.delete_chars,
            delete_bytes: res.delete_bytes,
            text: res.text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(engine: &mut VietKeyraEngine, input: &str) -> String {
        engine.reset_word();
        let mut buffer = String::new();
        for ch in input.chars() {
            match engine.process_char(ch) {
                KeyAction::PassThrough => buffer.push(ch),
                KeyAction::Commit { text } => buffer.push_str(&text),
                KeyAction::Replace {
                    delete_chars, text, ..
                } => {
                    for _ in 0..delete_chars {
                        buffer.pop();
                    }
                    buffer.push_str(&text);
                }
            }
        }
        buffer
    }

    #[test]
    fn test_mode_switch() {
        let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
        assert_eq!(feed(&mut engine, "toans"), "toán");

        engine.set_mode(InputMode::English);
        assert_eq!(feed(&mut engine, "toans"), "toans");

        engine.set_mode(InputMode::Vietnamese);
        assert_eq!(feed(&mut engine, "toans"), "toán");
    }

    #[test]
    fn test_backspace() {
        let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
        engine.process_char('a');
        assert_eq!(engine.current_word(), "a");
        let action = engine.handle_backspace();
        assert_eq!(
            action,
            KeyAction::Replace {
                delete_chars: 1,
                delete_bytes: 1,
                text: String::new()
            }
        );
        assert_eq!(engine.current_word(), "");

        let action2 = engine.handle_backspace();
        assert_eq!(action2, KeyAction::PassThrough);
    }

    #[test]
    fn test_backspace_history_cleared_on_empty() {
        let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);

        // Case 1: Modifier diacritic (2 keys -> 1 char)
        engine.process_char('a');
        engine.process_char('a');
        assert_eq!(engine.current_word(), "â");
        assert_eq!(engine.history_len(), 2);

        // Single backspace deletes 'â' and empties buffer
        let act = engine.handle_backspace();
        assert_eq!(
            act,
            KeyAction::Replace {
                delete_chars: 1,
                delete_bytes: 'â'.len_utf8(),
                text: String::new(),
            }
        );
        assert_eq!(engine.current_word(), "");
        assert!(engine.is_empty());
        assert_eq!(
            engine.history_len(),
            0,
            "History must be cleared when word becomes empty"
        );

        // Subsequent backspace is PassThrough and history remains 0
        let act2 = engine.handle_backspace();
        assert_eq!(act2, KeyAction::PassThrough);
        assert_eq!(engine.history_len(), 0);

        // Case 2: Multi-keystroke word with tone (5 keys -> 4 chars)
        for ch in "toans".chars() {
            engine.process_char(ch);
        }
        assert_eq!(engine.current_word(), "toán");
        assert_eq!(engine.history_len(), 5);

        // Backspace 4 times
        engine.handle_backspace(); // "toá"
        assert_eq!(engine.current_word(), "toá");
        engine.handle_backspace(); // "to"
        assert_eq!(engine.current_word(), "to");
        engine.handle_backspace(); // "t"
        assert_eq!(engine.current_word(), "t");
        engine.handle_backspace(); // "", history must clear
        assert_eq!(engine.current_word(), "");
        assert!(engine.is_empty());
        assert_eq!(
            engine.history_len(),
            0,
            "History must be cleared when multi-key word is fully deleted"
        );

        // Typing a new word starts completely fresh
        engine.process_char('a');
        assert_eq!(engine.current_word(), "a");
        assert_eq!(engine.history_len(), 1);
    }

    #[test]
    fn test_toggle_mode() {
        let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
        assert!(engine.get_mode().is_vietnamese());
        assert!(!engine.get_mode().is_english());

        engine.process_char('a');
        assert_eq!(engine.current_word(), "a");

        let mode = engine.toggle_mode();
        assert_eq!(mode, InputMode::English);
        assert!(engine.get_mode().is_english());
        assert_eq!(
            engine.current_word(),
            "",
            "Mode toggle must reset active word buffer"
        );

        let mode2 = engine.toggle_mode();
        assert_eq!(mode2, InputMode::Vietnamese);
        assert!(engine.get_mode().is_vietnamese());
    }
}
