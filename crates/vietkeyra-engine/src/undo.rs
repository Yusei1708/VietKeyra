//! Undo & Keystroke Step History State Machine
//!
//! Tracks historical transformations for dynamic backspacing and repeat-key reversions.

use crate::nfc::Tone;

/// Transformation category applied during a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepTransform {
    None,
    Append,
    DStroke,
    StandaloneW,
    ModifierA,
    ModifierE,
    ModifierO,
    ModifierW,
    ModifierUow,
    ModifierUoi,
    Tone { tone: Tone },
    Reverted { key: char },
}

/// A recorded transformation step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordStep {
    pub key: char,
    pub word_after: String,
    pub transform: StepTransform,
}

/// History of steps in the active word.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UndoHistory {
    steps: Vec<WordStep>,
}

impl UndoHistory {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn push(&mut self, step: WordStep) {
        self.steps.push(step);
    }

    pub fn pop(&mut self) -> Option<WordStep> {
        self.steps.pop()
    }

    pub fn last(&self) -> Option<&WordStep> {
        self.steps.last()
    }

    pub fn clear(&mut self) {
        self.steps.clear();
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_history() {
        let mut hist = UndoHistory::new();
        assert!(hist.is_empty());
        hist.push(WordStep {
            key: 'a',
            word_after: "a".to_string(),
            transform: StepTransform::Append,
        });
        assert_eq!(hist.len(), 1);
        assert_eq!(hist.last().unwrap().key, 'a');
        let popped = hist.pop().unwrap();
        assert_eq!(popped.word_after, "a");
        assert!(hist.is_empty());
    }
}
