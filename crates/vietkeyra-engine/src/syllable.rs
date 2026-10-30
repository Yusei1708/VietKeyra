//! Vietnamese Syllable Parser & Recomposer
//!
//! Decomposes a word string into canonical Vietnamese phonetic constituents:
//! [Onset] + [Nucleus Vowels] + [Coda] + [Tone]
//! and recomposes them into precomposed NFC strings with deterministic tone placement.

use crate::nfc::{self, BaseVowel, Case, Modifier, Tone};
use crate::tone::{self, TonePlacementStyle};

/// A parsed vowel slot within a syllable nucleus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedVowel {
    pub base: BaseVowel,
    pub modifier: Modifier,
    pub tone: Tone,
    pub case: Case,
    pub original_char: char,
}

/// Representation of a parsed Vietnamese syllable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSyllable {
    pub onset: String,
    pub vowels: Vec<ParsedVowel>,
    pub coda: String,
    pub existing_tone: Tone,
}

/// Parses a word string into its phonetic syllable components.
pub fn parse_syllable(word: &str) -> ParsedSyllable {
    let mut existing_tone = Tone::None;

    // Detect if any vowel currently carries a tone diacritic
    for ch in word.chars() {
        if let Some((_, _, tone, _)) = nfc::decompose_vowel(ch) {
            if tone != Tone::None {
                existing_tone = tone;
                break;
            }
        }
    }

    // Convert all vowels to their unaccented (tone = None) variants for structural parsing
    let mut plain_chars = Vec::with_capacity(word.len());
    for ch in word.chars() {
        if let Some((base, modifier, _, case)) = nfc::decompose_vowel(ch) {
            if let Some(plain_v) = nfc::compose_vowel(base, modifier, Tone::None, case) {
                plain_chars.push(plain_v);
            } else {
                plain_chars.push(ch);
            }
        } else {
            plain_chars.push(ch);
        }
    }
    let plain_str: String = plain_chars.into_iter().collect();
    let lower_str = plain_str.to_lowercase();

    let mut onset = String::new();
    let mut rest_idx = 0;

    // Detect multi-character and single onsets
    if lower_str.starts_with("ngh") {
        onset = plain_str[..3].to_string();
        rest_idx = 3;
    } else if lower_str.starts_with("qu") {
        onset = plain_str[..2].to_string();
        rest_idx = 2;
    } else if lower_str.starts_with("gi") {
        let chars_after_gi: Vec<char> = plain_str[2..].chars().collect();
        if !chars_after_gi.is_empty() && nfc::is_vowel(chars_after_gi[0]) {
            // gi is onset consonant [z] when followed by another vowel (già, giếng, giặt, gió, giúp)
            onset = plain_str[..2].to_string();
            rest_idx = 2;
        } else {
            // gi alone or followed by consonant: g is onset, i is nucleus vowel (gì, gìn)
            onset = plain_str[..1].to_string();
            rest_idx = 1;
        }
    } else {
        let digraphs = ["ng", "nh", "ch", "gh", "kh", "ph", "th", "tr"];
        let mut matched_digraph = false;
        for dg in digraphs {
            if lower_str.starts_with(dg) {
                onset = plain_str[..2].to_string();
                rest_idx = 2;
                matched_digraph = true;
                break;
            }
        }

        if !matched_digraph && !plain_str.is_empty() {
            let first_char = plain_str.chars().next().unwrap();
            if !nfc::is_vowel(first_char) {
                onset = first_char.to_string();
                rest_idx = first_char.len_utf8();
            }
        }
    }

    let rest = &plain_str[rest_idx..];
    let mut vowels = Vec::new();
    let mut coda_start = 0;

    for (byte_idx, ch) in rest.char_indices() {
        if let Some((base, modifier, _, case)) = nfc::decompose_vowel(ch) {
            vowels.push(ParsedVowel {
                base,
                modifier,
                tone: Tone::None,
                case,
                original_char: ch,
            });
            coda_start = byte_idx + ch.len_utf8();
        } else {
            break;
        }
    }

    let coda = rest[coda_start..].to_string();

    ParsedSyllable {
        onset,
        vowels,
        coda,
        existing_tone,
    }
}

/// Recomposes a syllable back into an NFC string, applying the tone to the correct target vowel.
pub fn compose_syllable(
    onset: &str,
    vowels: &[ParsedVowel],
    coda: &str,
    tone: Tone,
    style: TonePlacementStyle,
) -> String {
    if vowels.is_empty() {
        let mut res = String::with_capacity(onset.len() + coda.len());
        res.push_str(onset);
        res.push_str(coda);
        return res;
    }

    let base_vowels: Vec<BaseVowel> = vowels.iter().map(|v| v.base).collect();
    let has_coda = !coda.is_empty();
    let target_idx = tone::find_tone_target(&base_vowels, has_coda, style);

    let mut res = String::with_capacity(onset.len() + vowels.len() * 4 + coda.len());
    res.push_str(onset);

    for (idx, v) in vowels.iter().enumerate() {
        let t = if idx == target_idx { tone } else { Tone::None };
        if let Some(ch) = nfc::compose_vowel(v.base, v.modifier, t, v.case) {
            res.push(ch);
        } else {
            res.push(v.original_char);
        }
    }

    res.push_str(coda);
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let p = parse_syllable("toan");
        assert_eq!(p.onset, "t");
        assert_eq!(p.vowels.len(), 2);
        assert_eq!(p.coda, "n");
        assert_eq!(p.existing_tone, Tone::None);
    }

    #[test]
    fn test_parse_accented() {
        let p = parse_syllable("toán");
        assert_eq!(p.onset, "t");
        assert_eq!(p.vowels.len(), 2);
        assert_eq!(p.coda, "n");
        assert_eq!(p.existing_tone, Tone::Sac);
    }

    #[test]
    fn test_parse_special_onsets() {
        let p1 = parse_syllable("già");
        assert_eq!(p1.onset, "gi");
        assert_eq!(p1.vowels.len(), 1);
        assert_eq!(p1.vowels[0].base, BaseVowel::A);
        assert_eq!(p1.existing_tone, Tone::Huyen);

        let p2 = parse_syllable("gìn");
        assert_eq!(p2.onset, "g");
        assert_eq!(p2.vowels.len(), 1);
        assert_eq!(p2.vowels[0].base, BaseVowel::I);
        assert_eq!(p2.coda, "n");
        assert_eq!(p2.existing_tone, Tone::Huyen);

        let p3 = parse_syllable("quán");
        assert_eq!(p3.onset, "qu");
        assert_eq!(p3.vowels.len(), 1);
        assert_eq!(p3.vowels[0].base, BaseVowel::A);
        assert_eq!(p3.coda, "n");
        assert_eq!(p3.existing_tone, Tone::Sac);
    }

    #[test]
    fn test_compose_syllable_modern_vs_traditional() {
        let p = parse_syllable("hoa");
        let modern = compose_syllable(
            &p.onset,
            &p.vowels,
            &p.coda,
            Tone::Huyen,
            TonePlacementStyle::Modern,
        );
        assert_eq!(modern, "hoà");

        let trad = compose_syllable(
            &p.onset,
            &p.vowels,
            &p.coda,
            Tone::Huyen,
            TonePlacementStyle::Traditional,
        );
        assert_eq!(trad, "hòa");
    }
}
