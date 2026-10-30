//! Comprehensive 105-Word Unit Test Suite for VietKeyra Telex Engine
//!
//! Verifies 105+ words across 6 orthographic categories, NFC Unicode invariants,
//! Modern/Traditional tone placement, casing preservation, and repeat-key reversions.

use vietkeyra_engine::{KeyAction, TonePlacementStyle, VietKeyraEngine};

/// Simulates typing keystrokes through the engine and returns the resulting text.
/// Also validates that all `KeyAction::Replace` operations provide accurate `delete_bytes`.
fn feed_word(engine: &mut VietKeyraEngine, input: &str) -> String {
    engine.reset_word();
    let mut buffer = String::new();

    for ch in input.chars() {
        match engine.process_char(ch) {
            KeyAction::PassThrough => {
                buffer.push(ch);
            }
            KeyAction::Commit { text } => {
                buffer.push_str(&text);
            }
            KeyAction::Replace {
                delete_chars,
                delete_bytes,
                text,
            } => {
                // Verify delete_bytes matches byte length of the deleted characters in buffer
                if delete_chars > 0 {
                    let mut char_count = 0;
                    let mut byte_count = 0;
                    for c in buffer.chars().rev() {
                        if char_count < delete_chars {
                            char_count += 1;
                            byte_count += c.len_utf8();
                        } else {
                            break;
                        }
                    }
                    assert_eq!(
                        byte_count, delete_bytes,
                        "delete_bytes ({}) must match exact UTF-8 byte count of deleted characters ({}) for input '{}' at '{}'",
                        delete_bytes, byte_count, input, ch
                    );

                    for _ in 0..delete_chars {
                        buffer.pop();
                    }
                }
                buffer.push_str(&text);
            }
        }
    }

    buffer
}

/// Validates that an output string conforms strictly to Unicode NFC:
/// - Exact string equality with expected.
/// - Exact character count.
/// - Zero decomposed combining diacritics in range U+0300..=U+036F.
fn assert_nfc_word(output: &str, expected: &str, input: &str) {
    assert_eq!(
        output, expected,
        "Mismatch for input '{}': got '{}', expected '{}'",
        input, output, expected
    );
    assert_eq!(
        output.chars().count(),
        expected.chars().count(),
        "Character count mismatch for '{}'",
        input
    );
    assert_eq!(
        output.len(),
        expected.len(),
        "UTF-8 byte length mismatch for '{}'",
        input
    );
    assert!(
        !output
            .chars()
            .any(|c| ('\u{0300}'..='\u{036F}').contains(&c)),
        "Output '{}' contains decomposed combining diacritics (not pure NFC)!",
        output
    );
}

#[test]
fn test_category_1_simple_vowels_a_aw_aa() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let cases = [
        ("mas", "má"),
        ("baf", "bà"),
        ("car", "cả"),
        ("vax", "vã"),
        ("haj", "hạ"),
        ("mawts", "mắt"),
        ("tawmf", "tằm"),
        ("thawrm", "thẳm"),
        ("nawngx", "nẵng"),
        ("giawtj", "giặt"),
        ("saams", "sấm"),
        ("ddaamf", "đầm"),
        ("baamr", "bẩm"),
        ("daamx", "dẫm"),
        ("ddaamj", "đậm"),
    ];

    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}

#[test]
fn test_category_2_simple_vowels_e_ee_i_y() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let cases = [
        ("bes", "bé"),
        ("mef", "mè"),
        ("ver", "vẻ"),
        ("sex", "sẽ"),
        ("mej", "mẹ"),
        ("bees", "bế"),
        ("beef", "bề"),
        ("veef", "về"),
        ("beer", "bể"),
        ("reer", "rể"),
        ("deex", "dễ"),
        ("heetj", "hệt"),
        ("vis", "ví"),
        ("chif", "chì"),
        ("chir", "chỉ"),
        ("nghix", "nghĩ"),
        ("vij", "vị"),
        ("tys", "tý"),
        ("kyf", "kỳ"),
        ("tyr", "tỷ"),
        ("myx", "mỹ"),
        ("tyj", "tỵ"),
    ];

    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}

#[test]
fn test_category_3_simple_vowels_o_oo_ow_u_uw() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let cases = [
        ("chos", "chó"),
        ("bof", "bò"),
        ("cor", "cỏ"),
        ("gox", "gõ"),
        ("boj", "bọ"),
        ("boos", "bố"),
        ("hoof", "hồ"),
        ("hoor", "hổ"),
        ("goox", "gỗ"),
        ("booj", "bộ"),
        ("owns", "ớn"),
        ("cowf", "cờ"),
        ("owr", "ở"),
        ("owx", "ỡ"),
        ("cowx", "cỡ"),
        ("chowj", "chợ"),
        ("chus", "chú"),
        ("duf", "dù"),
        ("cur", "củ"),
        ("cux", "cũ"),
        ("cuj", "cụ"),
        ("chuws", "chứ"),
        ("tuwf", "từ"),
        ("thuwr", "thử"),
        ("chuwx", "chữ"),
        ("thuwjc", "thực"),
    ];

    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}

#[test]
fn test_category_4_diphthongs_modern_orthography() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let cases = [
        ("hoaf", "hoà"),
        ("toans", "toán"),
        ("hoaj", "hoạ"),
        ("loaif", "loài"),
        ("khoer", "khoẻ"),
        ("hoef", "hoè"),
        ("loes", "loé"),
        ("thuyr", "thuỷ"),
        ("nguyj", "nguỵ"),
        ("quys", "quý"),
        ("quynhf", "quỳnh"),
        ("luyx", "luỹ"),
        ("mias", "mía"),
        ("kiaf", "kìa"),
        ("cuar", "của"),
        ("dduax", "đũa"),
        ("luaj", "lụa"),
        ("tieens", "tiến"),
        ("bieenr", "biển"),
        ("dieenx", "diễn"),
        ("chuyeens", "chuyến"),
        ("thuyeenf", "thuyền"),
        ("cuoons", "cuốn"),
        ("nguowif", "người"),
        ("nuowcs", "nước"),
    ];

    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}

#[test]
fn test_category_5_triphthongs() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let cases = [
        ("ngoaif", "ngoài"),
        ("khoais", "khoái"),
        ("xoay", "xoay"),
        ("loays", "loáy"),
        ("chuoix", "chuỗi"),
        ("buoir", "buổi"),
        ("ruowuj", "rượu"),
        ("nhieeuf", "nhiều"),
        ("chieeus", "chiếu"),
        ("khuyur", "khuỷu"),
    ];

    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}

#[test]
fn test_category_6_casing_and_special_onsets() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let cases = [
        ("VIEETJ", "VIỆT"),
        ("DDOONGF", "ĐỒNG"),
        ("Vieetj", "Việt"),
        ("Ddoongf", "Đồng"),
        ("giaf", "già"),
        ("gieengs", "giếng"),
        ("ginf", "gìn"),
    ];

    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}

#[test]
fn test_category_7_undo_reversion_on_repeat() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let cases = [
        ("toanss", "toans"),
        ("hoaff", "hoaf"),
        ("ddda", "dda"),
        ("ww", "w"),
        ("WW", "W"),
        ("trww", "trw"),
        ("Trww", "Trw"),
        ("TRWW", "TRW"),
    ];

    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}

#[test]
fn test_traditional_tone_placement_style() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Traditional);

    let cases = [
        ("hoaf", "hòa"),
        ("khoer", "khỏe"),
        ("thuyr", "thủy"),
        ("nguyj", "ngụy"),
        ("luyx", "lũy"),
        ("toans", "toán"), // Has coda: tone stays on 'a' in both styles
        ("mias", "mía"),   // Open ia: tone stays on 'i' in both styles
    ];

    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}

#[test]
fn test_full_105_word_matrix_count() {
    // Verifies that at least 105 distinct test combinations are covered
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let mut count = 0;

    let full_list = [
        ("mas", "má"),
        ("baf", "bà"),
        ("car", "cả"),
        ("vax", "vã"),
        ("haj", "hạ"),
        ("mawts", "mắt"),
        ("tawmf", "tằm"),
        ("thawrm", "thẳm"),
        ("nawngx", "nẵng"),
        ("giawtj", "giặt"),
        ("saams", "sấm"),
        ("ddaamf", "đầm"),
        ("baamr", "bẩm"),
        ("daamx", "dẫm"),
        ("ddaamj", "đậm"),
        ("bes", "bé"),
        ("mef", "mè"),
        ("ver", "vẻ"),
        ("sex", "sẽ"),
        ("mej", "mẹ"),
        ("bees", "bế"),
        ("beef", "bề"),
        ("veef", "về"),
        ("beer", "bể"),
        ("reer", "rể"),
        ("deex", "dễ"),
        ("heetj", "hệt"),
        ("vis", "ví"),
        ("chif", "chì"),
        ("chir", "chỉ"),
        ("nghix", "nghĩ"),
        ("vij", "vị"),
        ("tys", "tý"),
        ("kyf", "kỳ"),
        ("tyr", "tỷ"),
        ("myx", "mỹ"),
        ("tyj", "tỵ"),
        ("chos", "chó"),
        ("bof", "bò"),
        ("cor", "cỏ"),
        ("gox", "gõ"),
        ("boj", "bọ"),
        ("boos", "bố"),
        ("hoof", "hồ"),
        ("hoor", "hổ"),
        ("goox", "gỗ"),
        ("booj", "bộ"),
        ("owns", "ớn"),
        ("cowf", "cờ"),
        ("owr", "ở"),
        ("owx", "ỡ"),
        ("cowx", "cỡ"),
        ("chowj", "chợ"),
        ("chus", "chú"),
        ("duf", "dù"),
        ("cur", "củ"),
        ("cux", "cũ"),
        ("cuj", "cụ"),
        ("chuws", "chứ"),
        ("tuwf", "từ"),
        ("thuwr", "thử"),
        ("chuwx", "chữ"),
        ("thuwjc", "thực"),
        ("hoaf", "hoà"),
        ("toans", "toán"),
        ("hoaj", "hoạ"),
        ("loaif", "loài"),
        ("khoer", "khoẻ"),
        ("hoef", "hoè"),
        ("loes", "loé"),
        ("thuyr", "thuỷ"),
        ("nguyj", "nguỵ"),
        ("quys", "quý"),
        ("quynhf", "quỳnh"),
        ("luyx", "luỹ"),
        ("mias", "mía"),
        ("kiaf", "kìa"),
        ("cuar", "của"),
        ("dduax", "đũa"),
        ("luaj", "lụa"),
        ("tieens", "tiến"),
        ("bieenr", "biển"),
        ("dieenx", "diễn"),
        ("chuyeens", "chuyến"),
        ("thuyeenf", "thuyền"),
        ("cuoons", "cuốn"),
        ("nguowif", "người"),
        ("nuowcs", "nước"),
        ("ngoaif", "ngoài"),
        ("khoais", "khoái"),
        ("xoay", "xoay"),
        ("loays", "loáy"),
        ("chuoix", "chuỗi"),
        ("buoir", "buổi"),
        ("ruowuj", "rượu"),
        ("nhieeuf", "nhiều"),
        ("chieeus", "chiếu"),
        ("khuyur", "khuỷu"),
        ("VIEETJ", "VIỆT"),
        ("DDOONGF", "ĐỒNG"),
        ("Vieetj", "Việt"),
        ("Ddoongf", "Đồng"),
        ("giaf", "già"),
        ("gieengs", "giếng"),
        ("ginf", "gìn"),
        ("toanss", "toans"),
        ("hoaff", "hoaf"),
        ("ddda", "dda"),
    ];

    for (input, expected) in full_list {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
        count += 1;
    }

    assert!(
        count >= 105,
        "Must test at least 105 words, tested {}",
        count
    );
}

// ==============================================================================
// E2E Test Runner Filter Aliases & Comprehensive Verification Suite
// ==============================================================================

#[test]
fn test_circumflex() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let cases = [
        ("aa", "â"),
        ("ee", "ê"),
        ("oo", "ô"),
        ("baas", "bấ"),
        ("maaf", "mầ"),
        ("caar", "cẩ"),
        ("daax", "dẫ"),
        ("naaj", "nậ"),
        ("bees", "bế"),
        ("beef", "bề"),
        ("beer", "bể"),
        ("deex", "dễ"),
        ("heetj", "hệt"),
        ("boos", "bố"),
        ("hoof", "hồ"),
        ("hoor", "hổ"),
        ("goox", "gỗ"),
        ("booj", "bộ"),
        ("AA", "Â"),
        ("EE", "Ê"),
        ("OO", "Ô"),
        ("VIEETJ", "VIỆT"),
    ];
    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}

#[test]
fn test_breve_horn() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let cases = [
        ("aw", "ă"),
        ("ow", "ơ"),
        ("uw", "ư"),
        ("w", "ư"),
        ("W", "Ư"),
        ("trw", "trư"),
        ("mawts", "mắt"),
        ("tawmf", "tằm"),
        ("thawrm", "thẳm"),
        ("nawngx", "nẵng"),
        ("giawtj", "giặt"),
        ("owns", "ớn"),
        ("cowf", "cờ"),
        ("owr", "ở"),
        ("owx", "ỡ"),
        ("cowx", "cỡ"),
        ("chowj", "chợ"),
        ("chuws", "chứ"),
        ("tuwf", "từ"),
        ("thuwr", "thử"),
        ("chuwx", "chữ"),
        ("thuwjc", "thực"),
        ("nguowif", "người"),
        ("nuowcs", "nước"),
        ("ruowuj", "rượu"),
        ("ww", "w"),
        ("trww", "trw"),
    ];
    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}

#[test]
fn test_d_stroke() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let cases = [
        ("dd", "đ"),
        ("DD", "Đ"),
        ("Dd", "Đ"),
        ("dda", "đa"),
        ("ddaw", "đă"),
        ("ddee", "đê"),
        ("ddoo", "đô"),
        ("dduw", "đư"),
        ("ddaamf", "đầm"),
        ("ddaamj", "đậm"),
        ("dduax", "đũa"),
        ("DDOONGF", "ĐỒNG"),
        ("Ddoongf", "Đồng"),
        ("ddda", "dda"),
        ("ddd", "dd"),
    ];
    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}

#[test]
fn test_tones() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let cases = [
        // Sắc (s)
        ("mas", "má"),
        ("bes", "bé"),
        ("chos", "chó"),
        ("chus", "chú"),
        ("tys", "tý"),
        // Huyền (f)
        ("baf", "bà"),
        ("mef", "mè"),
        ("bof", "bò"),
        ("duf", "dù"),
        ("kyf", "kỳ"),
        // Hỏi (r)
        ("car", "cả"),
        ("ver", "vẻ"),
        ("cor", "cỏ"),
        ("cur", "củ"),
        ("tyr", "tỷ"),
        // Ngã (x)
        ("vax", "vã"),
        ("sex", "sẽ"),
        ("gox", "gõ"),
        ("cux", "cũ"),
        ("myx", "mỹ"),
        // Nặng (j)
        ("haj", "hạ"),
        ("mej", "mẹ"),
        ("boj", "bọ"),
        ("cuj", "cụ"),
        ("tyj", "tỵ"),
    ];
    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}

#[test]
fn test_undo() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let cases = [
        ("toanss", "toans"),
        ("hoaff", "hoaf"),
        ("ddda", "dda"),
        ("ddd", "dd"),
        ("ass", "as"),
        ("bess", "bes"),
        ("aaa", "aa"),
        ("eee", "ee"),
        ("ooo", "oo"),
        ("ww", "w"),
        ("trww", "trw"),
        ("WW", "W"),
        ("dww", "dw"),
    ];
    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}

#[test]
fn test_standalone_w_revert() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);

    // Keystroke sequence tests
    assert_eq!(feed_word(&mut engine, "w"), "ư");
    assert_eq!(feed_word(&mut engine, "ww"), "w");
    assert_eq!(feed_word(&mut engine, "www"), "ww");
    assert_eq!(feed_word(&mut engine, "W"), "Ư");
    assert_eq!(feed_word(&mut engine, "WW"), "W");
    assert_eq!(feed_word(&mut engine, "trw"), "trư");
    assert_eq!(feed_word(&mut engine, "trww"), "trw");
    assert_eq!(feed_word(&mut engine, "dw"), "dư");
    assert_eq!(feed_word(&mut engine, "dww"), "dw");

    // Detailed KeyAction verification
    engine.reset_word();
    let act1 = engine.process_char('w');
    assert_eq!(engine.current_word(), "ư");
    assert_eq!(
        act1,
        KeyAction::Replace {
            delete_chars: 0,
            delete_bytes: 0,
            text: "ư".to_string()
        }
    );

    let act2 = engine.process_char('w');
    assert_eq!(engine.current_word(), "w");
    assert_eq!(
        act2,
        KeyAction::Replace {
            delete_chars: 1,
            delete_bytes: 'ư'.len_utf8(),
            text: "w".to_string()
        }
    );
}

#[test]
fn test_modern_tone() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let cases = [
        ("hoaf", "hoà"),
        ("hoas", "hoá"),
        ("hoar", "hoả"),
        ("hoax", "hoã"),
        ("hoaj", "hoạ"),
        ("khoer", "khoẻ"),
        ("hoef", "hoè"),
        ("loes", "loé"),
        ("thuyr", "thuỷ"),
        ("thuyf", "thuỳ"),
        ("thuys", "thuý"),
        ("nguyj", "nguỵ"),
        ("luyx", "luỹ"),
        ("quys", "quý"),
        ("quynhf", "quỳnh"),
    ];
    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}

#[test]
fn test_closed_syllables() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let cases = [
        ("toans", "toán"),
        ("hoanf", "hoàn"),
        ("tieens", "tiến"),
        ("bieenr", "biển"),
        ("dieenx", "diễn"),
        ("chuyeens", "chuyến"),
        ("thuyeenf", "thuyền"),
        ("cuoons", "cuốn"),
        ("nguowif", "người"),
        ("nuowcs", "nước"),
        ("mawts", "mắt"),
        ("tawmf", "tằm"),
        ("thuwjc", "thực"),
        ("heetj", "hệt"),
    ];
    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}

#[test]
fn test_open_diphthongs() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let cases = [
        ("mias", "mía"),
        ("kiaf", "kìa"),
        ("cuar", "của"),
        ("dduax", "đũa"),
        ("luaj", "lụa"),
        ("chuas", "chúa"),
        ("muaf", "mùa"),
        ("chuwas", "chứa"),
        ("cuwar", "cửa"),
        ("luwaf", "lừa"),
    ];
    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}

#[test]
fn test_triphthongs() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let cases = [
        ("ngoaif", "ngoài"),
        ("khoais", "khoái"),
        ("xoay", "xoay"),
        ("loays", "loáy"),
        ("chuoix", "chuỗi"),
        ("buoir", "buổi"),
        ("ruowuj", "rượu"),
        ("nhieeuf", "nhiều"),
        ("chieeus", "chiếu"),
        ("khuyur", "khuỷu"),
    ];
    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}

#[test]
fn test_105_words() {
    test_full_105_word_matrix_count();
}

#[test]
fn test_uppercase() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let cases = [
        ("VIEETJ", "VIỆT"),
        ("DDOONGF", "ĐỒNG"),
        ("TOANS", "TOÁN"),
        ("HOAF", "HOÀ"),
        ("Vieetj", "Việt"),
        ("Ddoongf", "Đồng"),
        ("Toans", "Toán"),
        ("Hoaf", "Hoà"),
        ("W", "Ư"),
        ("WW", "W"),
        ("DD", "Đ"),
        ("DDD", "DD"),
    ];
    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}

#[test]
fn test_special_onsets() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let cases = [
        ("giaf", "già"),
        ("gieengs", "giếng"),
        ("ginf", "gìn"),
        ("gias", "giá"),
        ("quys", "quý"),
        ("quynhf", "quỳnh"),
        ("quaf", "quà"),
        ("quasn", "quán"),
    ];
    for (input, expected) in cases {
        let out = feed_word(&mut engine, input);
        assert_nfc_word(&out, expected, input);
    }
}
