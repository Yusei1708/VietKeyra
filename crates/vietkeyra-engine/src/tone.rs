//! Vietnamese Tone Placement Logic (Modern BGD&ĐT vs Traditional)
//!
//! Provides deterministic target vowel selection for tone diacritics.

use crate::nfc::BaseVowel;

/// Style of tone placement for open diphthongs (oa, oe, uy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TonePlacementStyle {
    /// Modern standard (Quyết định 1989/QĐ-BGDĐT): hoà, thuỷ, khoè.
    #[default]
    Modern,
    /// Traditional / Classical style: hòa, thủy, khỏe.
    Traditional,
}

/// Identifies which vowel index in `vowels` should receive the tone mark.
///
/// # Arguments
/// - `vowels`: Slice of base vowels in the syllable nucleus.
/// - `has_coda`: Whether the syllable has a consonant coda (c, ch, m, n, ng, nh, p, t).
/// - `style`: Modern or Traditional tone placement style.
pub fn find_tone_target(vowels: &[BaseVowel], has_coda: bool, style: TonePlacementStyle) -> usize {
    match vowels.len() {
        0 => 0,
        1 => 0,
        2 => {
            if has_coda {
                // With a consonant coda (toán, hoàn, tiến, cuốn, mượn, huynh),
                // tone always falls on the second vowel (the main nucleus).
                1
            } else {
                // Open syllable (no consonant coda)
                let v0 = vowels[0];
                let v1 = vowels[1];

                // Group A: oa, oe, uy
                if (v0 == BaseVowel::O && v1 == BaseVowel::A)
                    || (v0 == BaseVowel::O && v1 == BaseVowel::E)
                    || (v0 == BaseVowel::U && v1 == BaseVowel::Y)
                {
                    match style {
                        TonePlacementStyle::Modern => 1,      // hoà, khoè, thuỷ
                        TonePlacementStyle::Traditional => 0, // hòa, khỏe, thủy
                    }
                } else if v0 == BaseVowel::U && (v1 == BaseVowel::E || v1 == BaseVowel::O) {
                    // uê (huế), uơ (thuở)
                    1
                } else {
                    // ia, ya, ua, ưa (mía, khuya, của, mứa)
                    // and off-glides ending in i, y, o, u (mái, máy, báo, cháu, ngửi, túi)
                    0
                }
            }
        }
        3 => {
            if has_coda {
                // uyê + coda (chuyến, thuyền, khuyên, duyệt, nguyễn)
                // The tone is on the 3rd vowel (ê, index 2).
                2
            } else {
                // Triphthongs: oai, oay, uôi, ươu, iêu, yêu, uyu, ươi
                // Tone is always placed on the middle vowel (index 1).
                1
            }
        }
        _ => vowels.len() / 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_vowel() {
        assert_eq!(
            find_tone_target(&[BaseVowel::A], false, TonePlacementStyle::Modern),
            0
        );
        assert_eq!(
            find_tone_target(&[BaseVowel::E], true, TonePlacementStyle::Modern),
            0
        );
    }

    #[test]
    fn test_two_vowels_with_coda() {
        // toán: o, a with coda n -> 1
        assert_eq!(
            find_tone_target(
                &[BaseVowel::O, BaseVowel::A],
                true,
                TonePlacementStyle::Modern
            ),
            1
        );
        assert_eq!(
            find_tone_target(
                &[BaseVowel::O, BaseVowel::A],
                true,
                TonePlacementStyle::Traditional
            ),
            1
        );
        // tiến: i, e with coda n -> 1
        assert_eq!(
            find_tone_target(
                &[BaseVowel::I, BaseVowel::E],
                true,
                TonePlacementStyle::Modern
            ),
            1
        );
        // huynh: u, y with coda nh -> 1
        assert_eq!(
            find_tone_target(
                &[BaseVowel::U, BaseVowel::Y],
                true,
                TonePlacementStyle::Modern
            ),
            1
        );
    }

    #[test]
    fn test_two_vowels_no_coda_modern_vs_traditional() {
        // oa (hoà / hòa)
        assert_eq!(
            find_tone_target(
                &[BaseVowel::O, BaseVowel::A],
                false,
                TonePlacementStyle::Modern
            ),
            1
        );
        assert_eq!(
            find_tone_target(
                &[BaseVowel::O, BaseVowel::A],
                false,
                TonePlacementStyle::Traditional
            ),
            0
        );

        // oe (khoè / khỏe)
        assert_eq!(
            find_tone_target(
                &[BaseVowel::O, BaseVowel::E],
                false,
                TonePlacementStyle::Modern
            ),
            1
        );
        assert_eq!(
            find_tone_target(
                &[BaseVowel::O, BaseVowel::E],
                false,
                TonePlacementStyle::Traditional
            ),
            0
        );

        // uy (thuỷ / thủy)
        assert_eq!(
            find_tone_target(
                &[BaseVowel::U, BaseVowel::Y],
                false,
                TonePlacementStyle::Modern
            ),
            1
        );
        assert_eq!(
            find_tone_target(
                &[BaseVowel::U, BaseVowel::Y],
                false,
                TonePlacementStyle::Traditional
            ),
            0
        );
    }

    #[test]
    fn test_two_vowels_no_coda_ia_ua_offglides() {
        // ia (mía)
        assert_eq!(
            find_tone_target(
                &[BaseVowel::I, BaseVowel::A],
                false,
                TonePlacementStyle::Modern
            ),
            0
        );
        assert_eq!(
            find_tone_target(
                &[BaseVowel::I, BaseVowel::A],
                false,
                TonePlacementStyle::Traditional
            ),
            0
        );

        // ua (của, lụa)
        assert_eq!(
            find_tone_target(
                &[BaseVowel::U, BaseVowel::A],
                false,
                TonePlacementStyle::Modern
            ),
            0
        );

        // ai (mái), ao (báo), ui (túi)
        assert_eq!(
            find_tone_target(
                &[BaseVowel::A, BaseVowel::I],
                false,
                TonePlacementStyle::Modern
            ),
            0
        );
        assert_eq!(
            find_tone_target(
                &[BaseVowel::A, BaseVowel::O],
                false,
                TonePlacementStyle::Modern
            ),
            0
        );
        assert_eq!(
            find_tone_target(
                &[BaseVowel::U, BaseVowel::I],
                false,
                TonePlacementStyle::Modern
            ),
            0
        );

        // uê (huế), uơ (thuở)
        assert_eq!(
            find_tone_target(
                &[BaseVowel::U, BaseVowel::E],
                false,
                TonePlacementStyle::Modern
            ),
            1
        );
        assert_eq!(
            find_tone_target(
                &[BaseVowel::U, BaseVowel::O],
                false,
                TonePlacementStyle::Modern
            ),
            1
        );
    }

    #[test]
    fn test_three_vowels_triphthongs() {
        // oai (ngoài, khoái) -> 1
        assert_eq!(
            find_tone_target(
                &[BaseVowel::O, BaseVowel::A, BaseVowel::I],
                false,
                TonePlacementStyle::Modern
            ),
            1
        );
        // uôi (chuối, buổi) -> 1
        assert_eq!(
            find_tone_target(
                &[BaseVowel::U, BaseVowel::O, BaseVowel::I],
                false,
                TonePlacementStyle::Modern
            ),
            1
        );
        // iêu (nhiều, chiếu) -> 1
        assert_eq!(
            find_tone_target(
                &[BaseVowel::I, BaseVowel::E, BaseVowel::U],
                false,
                TonePlacementStyle::Modern
            ),
            1
        );
        // uyu (khuỷu) -> 1
        assert_eq!(
            find_tone_target(
                &[BaseVowel::U, BaseVowel::Y, BaseVowel::U],
                false,
                TonePlacementStyle::Modern
            ),
            1
        );
    }

    #[test]
    fn test_three_vowels_with_coda() {
        // uyê + n (chuyến, thuyền) -> 2
        assert_eq!(
            find_tone_target(
                &[BaseVowel::U, BaseVowel::Y, BaseVowel::E],
                true,
                TonePlacementStyle::Modern
            ),
            2
        );
    }
}
