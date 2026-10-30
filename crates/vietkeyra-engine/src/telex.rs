//! Telex Transformation Rules Engine
//!
//! Handles Vietnamese Telex diacritic modifiers, free-tone placement, repeat-key undos,
//! and orthographic syllable reconstruction.

use crate::nfc::{BaseVowel, Case, Modifier, Tone};
use crate::syllable::{compose_syllable, parse_syllable, ParsedVowel};
use crate::tone::TonePlacementStyle;
use crate::undo::{StepTransform, WordStep};

/// Result of evaluating a Telex keystroke.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelexResult {
    pub new_word: String,
    pub transform: StepTransform,
    pub delete_chars: usize,
    pub delete_bytes: usize,
    pub text: String,
}

/// Evaluates a character key against the current word according to Telex rules.
pub fn process_telex_key(
    current_word: &str,
    ch: char,
    style: TonePlacementStyle,
    last_step: Option<&WordStep>,
) -> TelexResult {
    let k_low = ch.to_ascii_lowercase();

    // 1. D-stroke Handling
    if k_low == 'd' {
        // Repeat undo: đ -> dd or Đ -> DD / Dd
        if let Some(step) = last_step {
            if step.transform == StepTransform::DStroke {
                let is_upper = current_word.ends_with('Đ');
                let replacement = if ch.is_uppercase() {
                    "DD"
                } else if is_upper {
                    "Dd"
                } else {
                    "dd"
                };
                let mut new_word = current_word[..current_word.len() - 'đ'.len_utf8()].to_string();
                new_word.push_str(replacement);
                return TelexResult {
                    delete_chars: 1,
                    delete_bytes: 'đ'.len_utf8(),
                    text: replacement.to_string(),
                    new_word,
                    transform: StepTransform::Reverted { key: ch },
                };
            }
        }

        // Apply d-stroke
        if current_word.ends_with('d') || current_word.ends_with('D') {
            let is_upper = current_word.ends_with('D') || ch.is_uppercase();
            let stroke_char = if is_upper { 'Đ' } else { 'đ' };
            let mut new_word = current_word[..current_word.len() - 1].to_string();
            new_word.push(stroke_char);
            return TelexResult {
                delete_chars: 1,
                delete_bytes: 1,
                text: stroke_char.to_string(),
                new_word,
                transform: StepTransform::DStroke,
            };
        }
    }

    let parsed = parse_syllable(current_word);

    // 2. Tone Keys: s, f, r, x, j
    let target_tone = match k_low {
        's' => Some(Tone::Sac),
        'f' => Some(Tone::Huyen),
        'r' => Some(Tone::Hoi),
        'x' => Some(Tone::Nga),
        'j' => Some(Tone::Nang),
        _ => None,
    };

    if let Some(tone) = target_tone {
        if !parsed.vowels.is_empty() {
            // Check if last step was reverted with this exact key (user intentionally typing s, f, etc.)
            let is_repeated_raw = match last_step {
                Some(step) => {
                    matches!(step.transform, StepTransform::Reverted { key } if key == ch)
                }
                None => false,
            };

            if is_repeated_raw {
                // Raw append
            } else if parsed.existing_tone == tone {
                // Repeat undo: remove tone, append raw key
                let undone = compose_syllable(
                    &parsed.onset,
                    &parsed.vowels,
                    &parsed.coda,
                    Tone::None,
                    style,
                );
                let mut new_word = undone;
                new_word.push(ch);
                return TelexResult {
                    delete_chars: current_word.chars().count(),
                    delete_bytes: current_word.len(),
                    text: new_word.clone(),
                    new_word,
                    transform: StepTransform::Reverted { key: ch },
                };
            } else {
                // Apply or replace tone
                let new_word =
                    compose_syllable(&parsed.onset, &parsed.vowels, &parsed.coda, tone, style);
                return TelexResult {
                    delete_chars: current_word.chars().count(),
                    delete_bytes: current_word.len(),
                    text: new_word.clone(),
                    new_word,
                    transform: StepTransform::Tone { tone },
                };
            }
        }
    }

    // 3. Standalone 'w' (at start of word or after onset without vowels)
    if k_low == 'w' {
        if let Some(step) = last_step {
            if step.transform == StepTransform::StandaloneW {
                // Revert standalone w -> 'w'
                let stroke_len = if current_word.ends_with('Ư') {
                    'Ư'.len_utf8()
                } else {
                    'ư'.len_utf8()
                };
                let mut new_word = current_word[..current_word.len() - stroke_len].to_string();
                new_word.push(ch);
                return TelexResult {
                    delete_chars: 1,
                    delete_bytes: stroke_len,
                    text: ch.to_string(),
                    new_word,
                    transform: StepTransform::Reverted { key: ch },
                };
            }
        }

        // Apply standalone w -> 'ư' / 'Ư' (only if no vowels or coda yet, and word does not already end in 'w')
        if parsed.vowels.is_empty()
            && parsed.coda.is_empty()
            && !current_word.ends_with('w')
            && !current_word.ends_with('W')
        {
            let w_char = if ch.is_uppercase() { 'Ư' } else { 'ư' };
            let mut new_word = current_word.to_string();
            new_word.push(w_char);
            return TelexResult {
                delete_chars: 0,
                delete_bytes: 0,
                text: w_char.to_string(),
                new_word,
                transform: StepTransform::StandaloneW,
            };
        }
    }

    // 4. Vowel Modifiers: a, e, o, w
    if parsed.coda.is_empty() && !parsed.vowels.is_empty() {
        let last_v = parsed.vowels.last().unwrap();

        // Modifier 'a': aa -> â
        if k_low == 'a' {
            if let Some(step) = last_step {
                if step.transform == StepTransform::ModifierA {
                    // Undo â -> aa
                    let mut new_v = parsed.vowels.clone();
                    new_v.last_mut().unwrap().modifier = Modifier::None;
                    let base_str = compose_syllable(
                        &parsed.onset,
                        &new_v,
                        &parsed.coda,
                        parsed.existing_tone,
                        style,
                    );
                    let mut new_word = base_str;
                    new_word.push(ch);
                    return TelexResult {
                        delete_chars: current_word.chars().count(),
                        delete_bytes: current_word.len(),
                        text: new_word.clone(),
                        new_word,
                        transform: StepTransform::Reverted { key: ch },
                    };
                }
            }

            if last_v.base == BaseVowel::A && last_v.modifier == Modifier::None {
                let mut new_v = parsed.vowels.clone();
                let slot = new_v.last_mut().unwrap();
                slot.modifier = Modifier::Circumflex;
                if ch.is_uppercase() {
                    slot.case = Case::Upper;
                }
                let new_word = compose_syllable(
                    &parsed.onset,
                    &new_v,
                    &parsed.coda,
                    parsed.existing_tone,
                    style,
                );
                return TelexResult {
                    delete_chars: current_word.chars().count(),
                    delete_bytes: current_word.len(),
                    text: new_word.clone(),
                    new_word,
                    transform: StepTransform::ModifierA,
                };
            }
        }

        // Modifier 'e': ee -> ê
        if k_low == 'e' {
            if let Some(step) = last_step {
                if step.transform == StepTransform::ModifierE {
                    // Undo ê -> ee
                    let mut new_v = parsed.vowels.clone();
                    new_v.last_mut().unwrap().modifier = Modifier::None;
                    let base_str = compose_syllable(
                        &parsed.onset,
                        &new_v,
                        &parsed.coda,
                        parsed.existing_tone,
                        style,
                    );
                    let mut new_word = base_str;
                    new_word.push(ch);
                    return TelexResult {
                        delete_chars: current_word.chars().count(),
                        delete_bytes: current_word.len(),
                        text: new_word.clone(),
                        new_word,
                        transform: StepTransform::Reverted { key: ch },
                    };
                }
            }

            if last_v.base == BaseVowel::E && last_v.modifier == Modifier::None {
                let mut new_v = parsed.vowels.clone();
                let slot = new_v.last_mut().unwrap();
                slot.modifier = Modifier::Circumflex;
                if ch.is_uppercase() {
                    slot.case = Case::Upper;
                }
                let new_word = compose_syllable(
                    &parsed.onset,
                    &new_v,
                    &parsed.coda,
                    parsed.existing_tone,
                    style,
                );
                return TelexResult {
                    delete_chars: current_word.chars().count(),
                    delete_bytes: current_word.len(),
                    text: new_word.clone(),
                    new_word,
                    transform: StepTransform::ModifierE,
                };
            }
        }

        // Modifier 'o': oo -> ô
        if k_low == 'o' {
            if let Some(step) = last_step {
                if step.transform == StepTransform::ModifierO {
                    // Undo ô -> oo
                    let mut new_v = parsed.vowels.clone();
                    new_v.last_mut().unwrap().modifier = Modifier::None;
                    let base_str = compose_syllable(
                        &parsed.onset,
                        &new_v,
                        &parsed.coda,
                        parsed.existing_tone,
                        style,
                    );
                    let mut new_word = base_str;
                    new_word.push(ch);
                    return TelexResult {
                        delete_chars: current_word.chars().count(),
                        delete_bytes: current_word.len(),
                        text: new_word.clone(),
                        new_word,
                        transform: StepTransform::Reverted { key: ch },
                    };
                }
            }

            if last_v.base == BaseVowel::O && last_v.modifier == Modifier::None {
                let mut new_v = parsed.vowels.clone();
                let slot = new_v.last_mut().unwrap();
                slot.modifier = Modifier::Circumflex;
                if ch.is_uppercase() {
                    slot.case = Case::Upper;
                }
                let new_word = compose_syllable(
                    &parsed.onset,
                    &new_v,
                    &parsed.coda,
                    parsed.existing_tone,
                    style,
                );
                return TelexResult {
                    delete_chars: current_word.chars().count(),
                    delete_bytes: current_word.len(),
                    text: new_word.clone(),
                    new_word,
                    transform: StepTransform::ModifierO,
                };
            }
        }

        // Modifier 'w':
        if k_low == 'w' {
            // Check uo + w -> ươ
            if parsed.vowels.len() >= 2 {
                let n = parsed.vowels.len();
                let v_prev = &parsed.vowels[n - 2];
                let v_last = &parsed.vowels[n - 1];

                if v_prev.base == BaseVowel::U && v_last.base == BaseVowel::O {
                    if let Some(step) = last_step {
                        if step.transform == StepTransform::ModifierUow {
                            // Undo ươ -> uow
                            let mut new_v = parsed.vowels.clone();
                            new_v[n - 2].modifier = Modifier::None;
                            new_v[n - 1].modifier = Modifier::None;
                            let base_str = compose_syllable(
                                &parsed.onset,
                                &new_v,
                                &parsed.coda,
                                parsed.existing_tone,
                                style,
                            );
                            let mut new_word = base_str;
                            new_word.push(ch);
                            return TelexResult {
                                delete_chars: current_word.chars().count(),
                                delete_bytes: current_word.len(),
                                text: new_word.clone(),
                                new_word,
                                transform: StepTransform::Reverted { key: ch },
                            };
                        }
                    }

                    let mut new_v = parsed.vowels.clone();
                    new_v[n - 2].modifier = Modifier::Horn;
                    new_v[n - 1].modifier = Modifier::Horn;
                    if ch.is_uppercase() {
                        new_v[n - 1].case = Case::Upper;
                    }
                    let new_word = compose_syllable(
                        &parsed.onset,
                        &new_v,
                        &parsed.coda,
                        parsed.existing_tone,
                        style,
                    );
                    return TelexResult {
                        delete_chars: current_word.chars().count(),
                        delete_bytes: current_word.len(),
                        text: new_word.clone(),
                        new_word,
                        transform: StepTransform::ModifierUow,
                    };
                }
            }

            // a + w -> ă
            if last_v.base == BaseVowel::A {
                if let Some(step) = last_step {
                    if step.transform == StepTransform::ModifierW {
                        // Undo ă -> aw
                        let mut new_v = parsed.vowels.clone();
                        new_v.last_mut().unwrap().modifier = Modifier::None;
                        let base_str = compose_syllable(
                            &parsed.onset,
                            &new_v,
                            &parsed.coda,
                            parsed.existing_tone,
                            style,
                        );
                        let mut new_word = base_str;
                        new_word.push(ch);
                        return TelexResult {
                            delete_chars: current_word.chars().count(),
                            delete_bytes: current_word.len(),
                            text: new_word.clone(),
                            new_word,
                            transform: StepTransform::Reverted { key: ch },
                        };
                    }
                }

                if last_v.modifier == Modifier::None {
                    let mut new_v = parsed.vowels.clone();
                    let slot = new_v.last_mut().unwrap();
                    slot.modifier = Modifier::Breve;
                    if ch.is_uppercase() {
                        slot.case = Case::Upper;
                    }
                    let new_word = compose_syllable(
                        &parsed.onset,
                        &new_v,
                        &parsed.coda,
                        parsed.existing_tone,
                        style,
                    );
                    return TelexResult {
                        delete_chars: current_word.chars().count(),
                        delete_bytes: current_word.len(),
                        text: new_word.clone(),
                        new_word,
                        transform: StepTransform::ModifierW,
                    };
                }
            }

            // o + w -> ơ
            if last_v.base == BaseVowel::O {
                if let Some(step) = last_step {
                    if step.transform == StepTransform::ModifierW {
                        // Undo ơ -> ow
                        let mut new_v = parsed.vowels.clone();
                        new_v.last_mut().unwrap().modifier = Modifier::None;
                        let base_str = compose_syllable(
                            &parsed.onset,
                            &new_v,
                            &parsed.coda,
                            parsed.existing_tone,
                            style,
                        );
                        let mut new_word = base_str;
                        new_word.push(ch);
                        return TelexResult {
                            delete_chars: current_word.chars().count(),
                            delete_bytes: current_word.len(),
                            text: new_word.clone(),
                            new_word,
                            transform: StepTransform::Reverted { key: ch },
                        };
                    }
                }

                if last_v.modifier == Modifier::None {
                    let mut new_v = parsed.vowels.clone();
                    let slot = new_v.last_mut().unwrap();
                    slot.modifier = Modifier::Horn;
                    if ch.is_uppercase() {
                        slot.case = Case::Upper;
                    }
                    let new_word = compose_syllable(
                        &parsed.onset,
                        &new_v,
                        &parsed.coda,
                        parsed.existing_tone,
                        style,
                    );
                    return TelexResult {
                        delete_chars: current_word.chars().count(),
                        delete_bytes: current_word.len(),
                        text: new_word.clone(),
                        new_word,
                        transform: StepTransform::ModifierW,
                    };
                }
            }

            // u + w -> ư
            if last_v.base == BaseVowel::U {
                if let Some(step) = last_step {
                    if step.transform == StepTransform::ModifierW {
                        // Undo ư -> uw
                        let mut new_v = parsed.vowels.clone();
                        new_v.last_mut().unwrap().modifier = Modifier::None;
                        let base_str = compose_syllable(
                            &parsed.onset,
                            &new_v,
                            &parsed.coda,
                            parsed.existing_tone,
                            style,
                        );
                        let mut new_word = base_str;
                        new_word.push(ch);
                        return TelexResult {
                            delete_chars: current_word.chars().count(),
                            delete_bytes: current_word.len(),
                            text: new_word.clone(),
                            new_word,
                            transform: StepTransform::Reverted { key: ch },
                        };
                    }
                }

                if last_v.modifier == Modifier::None {
                    let mut new_v = parsed.vowels.clone();
                    let slot = new_v.last_mut().unwrap();
                    slot.modifier = Modifier::Horn;
                    if ch.is_uppercase() {
                        slot.case = Case::Upper;
                    }
                    let new_word = compose_syllable(
                        &parsed.onset,
                        &new_v,
                        &parsed.coda,
                        parsed.existing_tone,
                        style,
                    );
                    return TelexResult {
                        delete_chars: current_word.chars().count(),
                        delete_bytes: current_word.len(),
                        text: new_word.clone(),
                        new_word,
                        transform: StepTransform::ModifierW,
                    };
                }
            }
        }
    }

    // 5. Auto-orthography: u + o + i -> u + ô + i (for uôi triphthongs)
    if k_low == 'i' && parsed.coda.is_empty() && parsed.vowels.len() >= 2 {
        let n = parsed.vowels.len();
        if parsed.vowels[n - 2].base == BaseVowel::U
            && parsed.vowels[n - 2].modifier == Modifier::None
            && parsed.vowels[n - 1].base == BaseVowel::O
            && parsed.vowels[n - 1].modifier == Modifier::None
        {
            let mut new_v = parsed.vowels.clone();
            new_v[n - 1].modifier = Modifier::Circumflex;
            new_v.push(ParsedVowel {
                base: BaseVowel::I,
                modifier: Modifier::None,
                tone: Tone::None,
                case: if ch.is_uppercase() {
                    Case::Upper
                } else {
                    Case::Lower
                },
                original_char: ch,
            });
            let new_word = compose_syllable(
                &parsed.onset,
                &new_v,
                &parsed.coda,
                parsed.existing_tone,
                style,
            );
            return TelexResult {
                delete_chars: current_word.chars().count(),
                delete_bytes: current_word.len(),
                text: new_word.clone(),
                new_word,
                transform: StepTransform::ModifierUoi,
            };
        }
    }

    // 6. Default: Append character
    let mut raw_new_word = current_word.to_string();
    raw_new_word.push(ch);

    if parsed.existing_tone != Tone::None {
        // Re-evaluate tone position with the new character
        let new_parsed = parse_syllable(&raw_new_word);
        let new_word = compose_syllable(
            &new_parsed.onset,
            &new_parsed.vowels,
            &new_parsed.coda,
            parsed.existing_tone,
            style,
        );
        TelexResult {
            delete_chars: current_word.chars().count(),
            delete_bytes: current_word.len(),
            text: new_word.clone(),
            new_word,
            transform: StepTransform::Append,
        }
    } else {
        TelexResult {
            delete_chars: 0,
            delete_bytes: 0,
            text: ch.to_string(),
            new_word: raw_new_word,
            transform: StepTransform::Append,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telex_simple() {
        let r1 = process_telex_key("", 'm', TonePlacementStyle::Modern, None);
        assert_eq!(r1.new_word, "m");

        let step1 = WordStep {
            key: 'm',
            word_after: r1.new_word.clone(),
            transform: r1.transform,
        };
        let r2 = process_telex_key(&r1.new_word, 'a', TonePlacementStyle::Modern, Some(&step1));
        assert_eq!(r2.new_word, "ma");

        let step2 = WordStep {
            key: 'a',
            word_after: r2.new_word.clone(),
            transform: r2.transform,
        };
        let r3 = process_telex_key(&r2.new_word, 's', TonePlacementStyle::Modern, Some(&step2));
        assert_eq!(r3.new_word, "má");
    }

    #[test]
    fn test_telex_repeat_undo() {
        let r1 = process_telex_key("toan", 's', TonePlacementStyle::Modern, None);
        assert_eq!(r1.new_word, "toán");

        let step1 = WordStep {
            key: 's',
            word_after: r1.new_word.clone(),
            transform: r1.transform,
        };
        let r2 = process_telex_key(&r1.new_word, 's', TonePlacementStyle::Modern, Some(&step1));
        assert_eq!(r2.new_word, "toans");
    }

    #[test]
    fn test_telex_standalone_w_repeat_reversion() {
        // Test ww -> w
        let r1 = process_telex_key("", 'w', TonePlacementStyle::Modern, None);
        assert_eq!(r1.new_word, "ư");
        assert_eq!(r1.transform, StepTransform::StandaloneW);

        let step1 = WordStep {
            key: 'w',
            word_after: r1.new_word.clone(),
            transform: r1.transform,
        };
        let r2 = process_telex_key(&r1.new_word, 'w', TonePlacementStyle::Modern, Some(&step1));
        assert_eq!(r2.new_word, "w");
        assert_eq!(r2.delete_chars, 1);
        assert_eq!(r2.delete_bytes, 'ư'.len_utf8());
        assert_eq!(r2.text, "w");
        assert_eq!(r2.transform, StepTransform::Reverted { key: 'w' });

        // Test trww -> trw
        let r3 = process_telex_key("tr", 'w', TonePlacementStyle::Modern, None);
        assert_eq!(r3.new_word, "trư");
        assert_eq!(r3.transform, StepTransform::StandaloneW);

        let step3 = WordStep {
            key: 'w',
            word_after: r3.new_word.clone(),
            transform: r3.transform,
        };
        let r4 = process_telex_key(&r3.new_word, 'w', TonePlacementStyle::Modern, Some(&step3));
        assert_eq!(r4.new_word, "trw");
        assert_eq!(r4.delete_chars, 1);
        assert_eq!(r4.delete_bytes, 'ư'.len_utf8());
        assert_eq!(r4.text, "w");
        assert_eq!(r4.transform, StepTransform::Reverted { key: 'w' });

        // Test WW -> W
        let r5 = process_telex_key("", 'W', TonePlacementStyle::Modern, None);
        assert_eq!(r5.new_word, "Ư");
        let step5 = WordStep {
            key: 'W',
            word_after: r5.new_word.clone(),
            transform: r5.transform,
        };
        let r6 = process_telex_key(&r5.new_word, 'W', TonePlacementStyle::Modern, Some(&step5));
        assert_eq!(r6.new_word, "W");
        assert_eq!(r6.delete_bytes, 'Ư'.len_utf8());
    }
}
