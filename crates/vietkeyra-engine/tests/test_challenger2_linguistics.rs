//! Adversarial Empirical Challenge Suite for VietKeyra Telex Engine (Milestone 1)
//!
//! Stress-tests linguistic accuracy, Modern vs Traditional tone placements,
//! special onsets (qu, gi, ngh), uppercase/titlecase/mixed-case preservation,
//! repeat-key reversions, free-tone typing order, and Wayland UTF-8 delete_bytes calculations.

use vietkeyra_engine::{KeyAction, TonePlacementStyle, VietKeyraEngine};

/// Independent test oracle that tracks typed text and validates UTF-8 delete_bytes calculations.
struct TestHarness {
    engine: VietKeyraEngine,
    buffer: String,
}

impl TestHarness {
    fn new(style: TonePlacementStyle) -> Self {
        Self {
            engine: VietKeyraEngine::new(style),
            buffer: String::new(),
        }
    }

    fn feed(&mut self, input: &str) -> String {
        self.engine.reset_word();
        self.buffer.clear();

        for ch in input.chars() {
            let action = self.engine.process_char(ch);
            match action {
                KeyAction::PassThrough => {
                    self.buffer.push(ch);
                }
                KeyAction::Commit { text } => {
                    self.buffer.push_str(&text);
                }
                KeyAction::Replace {
                    delete_chars,
                    delete_bytes,
                    text,
                } => {
                    // EMPIRICAL ORACLE: Verify delete_bytes strictly matches deleted UTF-8 slice
                    if delete_chars > 0 {
                        let mut byte_count = 0;
                        let mut char_count = 0;
                        for c in self.buffer.chars().rev() {
                            if char_count < delete_chars {
                                char_count += 1;
                                byte_count += c.len_utf8();
                            } else {
                                break;
                            }
                        }
                        assert_eq!(
                            char_count, delete_chars,
                            "Buffer underflow: requested delete_chars {} exceeds buffer char count {}",
                            delete_chars, self.buffer.chars().count()
                        );
                        assert_eq!(
                            byte_count, delete_bytes,
                            "CRITICAL CONTRACT VIOLATION: delete_bytes ({}) != actual UTF-8 bytes ({}) for input '{}' at '{}'",
                            delete_bytes, byte_count, input, ch
                        );

                        for _ in 0..delete_chars {
                            self.buffer.pop();
                        }
                    } else {
                        assert_eq!(
                            delete_bytes, 0,
                            "delete_bytes must be 0 when delete_chars is 0"
                        );
                    }
                    self.buffer.push_str(&text);
                }
            }
        }

        // Verify NFC invariants
        assert!(
            !self
                .buffer
                .chars()
                .any(|c| ('\u{0300}'..='\u{036F}').contains(&c)),
            "Output contains decomposed combining diacritics: '{}'",
            self.buffer
        );

        self.buffer.clone()
    }
}

#[test]
fn test_challenge_special_onsets_qu() {
    let mut harness = TestHarness::new(TonePlacementStyle::Modern);

    let qu_words = [
        ("quans", "quán"),
        ("quys", "quý"),
        ("quyeets", "quyết"),
        ("quynhf", "quỳnh"),
        ("quaanf", "quần"),
        ("quaanj", "quận"),
        ("quoocs", "quốc"),
        ("quow", "quơ"),
        ("quowr", "quở"),
        ("queoj", "quẹo"),
        ("quais", "quái"),
        ("quaf", "quà"),
        ("quax", "quã"),
        ("quaj", "quạ"),
    ];

    for (input, expected) in qu_words {
        let actual = harness.feed(input);
        assert_eq!(
            actual, expected,
            "Failed special onset 'qu' for input '{}'",
            input
        );
    }
}

#[test]
fn test_challenge_special_onsets_gi() {
    let mut harness = TestHarness::new(TonePlacementStyle::Modern);

    let gi_words = [
        ("giaf", "già"),
        ("giawtj", "giặt"),
        ("gieengs", "giếng"),
        ("ginf", "gìn"),
        ("gif", "gì"),
        ("gios", "gió"),
        ("giups", "giúp"),
        ("giuwax", "giữa"),
        ("giais", "giái"),
        ("giauf", "giàu"),
        ("giuowng", "giương"),
        ("gieetj", "giệt"),
        ("gieej", "giệ"),
    ];

    for (input, expected) in gi_words {
        let actual = harness.feed(input);
        assert_eq!(
            actual, expected,
            "Failed special onset 'gi' for input '{}'",
            input
        );
    }
}

#[test]
fn test_challenge_special_onsets_ngh() {
    let mut harness = TestHarness::new(TonePlacementStyle::Modern);

    let ngh_words = [
        ("nghix", "nghĩ"),
        ("ngheej", "nghệ"),
        ("nghieeng", "nghiêng"),
        ("ngheof", "nghèo"),
        ("nghieem", "nghiêm"),
        ("nghichj", "nghịch"),
        ("nghir", "nghỉ"),
        ("nghes", "nghé"),
    ];

    for (input, expected) in ngh_words {
        let actual = harness.feed(input);
        assert_eq!(
            actual, expected,
            "Failed special onset 'ngh' for input '{}'",
            input
        );
    }
}

#[test]
fn test_challenge_modern_vs_traditional_contrast() {
    let mut modern = TestHarness::new(TonePlacementStyle::Modern);
    let mut traditional = TestHarness::new(TonePlacementStyle::Traditional);

    // Open diphthongs oa, oe, uy MUST differ between Modern and Traditional
    let contrasting_pairs = [
        ("hoaf", "hoà", "hòa"),
        ("khoer", "khoẻ", "khỏe"),
        ("thuyr", "thuỷ", "thủy"),
        ("nguyj", "nguỵ", "ngụy"),
        ("luyx", "luỹ", "lũy"),
        ("tuyf", "tuỳ", "tùy"),
        ("xoef", "xoè", "xòe"),
        ("loef", "loè", "lòe"),
        ("thuys", "thuý", "thúy"),
        ("toas", "toá", "tóa"),
    ];

    for (input, exp_modern, exp_trad) in contrasting_pairs {
        let act_modern = modern.feed(input);
        assert_eq!(
            act_modern, exp_modern,
            "Modern style mismatch for '{}'",
            input
        );

        let act_trad = traditional.feed(input);
        assert_eq!(
            act_trad, exp_trad,
            "Traditional style mismatch for '{}'",
            input
        );
    }
}

#[test]
fn test_challenge_style_invariant_syllables() {
    let mut modern = TestHarness::new(TonePlacementStyle::Modern);
    let mut traditional = TestHarness::new(TonePlacementStyle::Traditional);

    // Syllables with coda, triphthongs, and non-Group-A syllables MUST NOT differ between styles
    let invariants = [
        ("toans", "toán"),
        ("hoanf", "hoàn"),
        ("huynhf", "huỳnh"),
        ("tieens", "tiến"),
        ("cuoons", "cuốn"),
        ("quys", "quý"), // 'u' is in onset consonant 'qu', NEVER 'qúy'
        ("quans", "quán"),
        ("giaf", "già"),
        ("gios", "gió"),
        ("ngoaif", "ngoài"),
        ("xoays", "xoáy"),
        ("chuoix", "chuỗi"),
        ("ruowuj", "rượu"),
        ("nguowif", "người"),
        ("nuowcs", "nước"),
        ("chieeus", "chiếu"),
        ("nhieeuf", "nhiều"),
        ("chuyeens", "chuyến"),
        ("thuyeenf", "thuyền"),
        ("khuyur", "khuỷu"),
        ("mias", "mía"),
        ("cuar", "của"),
    ];

    for (input, expected) in invariants {
        let act_mod = modern.feed(input);
        assert_eq!(act_mod, expected, "Modern invariant failed for '{}'", input);

        let act_trad = traditional.feed(input);
        assert_eq!(
            act_trad, expected,
            "Traditional invariant failed for '{}'",
            input
        );
    }
}

#[test]
fn test_challenge_casing_preservation() {
    let mut harness = TestHarness::new(TonePlacementStyle::Modern);

    // UPPERCASE
    let uppercase_words = [
        ("VIEETJ", "VIỆT"),
        ("DDOONGF", "ĐỒNG"),
        ("NAI", "NAI"),
        ("DDUOWFNG", "ĐƯỜNG"),
        ("XAX", "XÃ"),
        ("QUYEETS", "QUYẾT"),
        ("TAAM", "TÂM"),
        ("GIEENGS", "GIẾNG"),
        ("NUOWCS", "NƯỚC"),
        ("THUYR", "THUỶ"),
        ("DDIEENJ", "ĐIỆN"),
        ("HOAF", "HOÀ"),
        ("BINHF", "BÌNH"),
    ];

    for (input, expected) in uppercase_words {
        let actual = harness.feed(input);
        assert_eq!(actual, expected, "Uppercase failed for '{}'", input);
    }

    // Titlecase
    let titlecase_words = [
        ("Vieetj", "Việt"),
        ("Ddoongf", "Đồng"),
        ("Dduowfng", "Đường"),
        ("Xax", "Xã"),
        ("Quyeets", "Quyết"),
        ("Gieengs", "Giếng"),
        ("Thuyr", "Thuỷ"),
        ("Hoaf", "Hoà"),
    ];

    for (input, expected) in titlecase_words {
        let actual = harness.feed(input);
        assert_eq!(actual, expected, "Titlecase failed for '{}'", input);
    }

    // D-stroke casing variations
    assert_eq!(harness.feed("DD"), "Đ");
    assert_eq!(harness.feed("Dd"), "Đ");
    assert_eq!(harness.feed("dd"), "đ");
    assert_eq!(harness.feed("dD"), "Đ");
}

#[test]
fn test_challenge_repeat_key_reversion() {
    let mut harness = TestHarness::new(TonePlacementStyle::Modern);

    let repeat_cases = [
        // Tone repetition
        ("toanss", "toans"),
        ("hoaff", "hoaf"),
        ("mass", "mas"),
        ("mejj", "mej"),
        ("curr", "cur"),
        ("cuxx", "cux"),
        // Multi-repeat tones
        ("toansss", "toanss"),
        ("hoafff", "hoaff"),
        // Modifier repetition
        ("aaa", "aa"),
        ("aww", "aw"),
        ("eee", "ee"),
        ("ooo", "oo"),
        ("uww", "uw"),
        ("oww", "ow"),
        ("uoww", "uow"),
        // D-stroke repetition
        ("ddda", "dda"),
        ("DDD", "DD"),
        ("Ddd", "Dd"),
        ("ddd", "dd"),
        // Standalone w: ww -> w, WW -> W, trww -> trw
        ("ww", "w"),
        ("WW", "W"),
        ("trww", "trw"),
        // Additional repeat edge cases
        ("aaaa", "aâ"),
        ("toanssss", "toánss"),
    ];

    for (input, expected) in repeat_cases {
        let actual = harness.feed(input);
        assert_eq!(actual, expected, "Repeat undo failed for '{}'", input);
    }
}

#[test]
fn test_challenge_free_tone_typing_order() {
    let mut harness = TestHarness::new(TonePlacementStyle::Modern);

    // Free tone typed immediately after vowel or mid-word
    let free_tone_cases = [
        ("toasn", "toán"),
        ("hoafn", "hoàn"),
        ("thuyeefn", "thuyền"),
        ("tieesn", "tiến"),
        ("cuoosn", "cuốn"),
        ("bieern", "biển"),
        ("dduowfng", "đường"),
        ("nguyeexn", "nguyễn"),
    ];

    for (input, expected) in free_tone_cases {
        let actual = harness.feed(input);
        assert_eq!(actual, expected, "Free-tone typing failed for '{}'", input);
    }
}

#[test]
fn test_challenge_backspace_oracle() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let mut buffer = String::new();

    // Type "toán"
    for ch in "toans".chars() {
        if let KeyAction::Replace {
            delete_chars,
            delete_bytes,
            text,
        } = engine.process_char(ch)
        {
            let mut byte_count = 0;
            let mut char_count = 0;
            for c in buffer.chars().rev() {
                if char_count < delete_chars {
                    char_count += 1;
                    byte_count += c.len_utf8();
                } else {
                    break;
                }
            }
            assert_eq!(byte_count, delete_bytes);
            for _ in 0..delete_chars {
                buffer.pop();
            }
            buffer.push_str(&text);
        }
    }
    assert_eq!(buffer, "toán");

    // Backspace: removes 'n'
    let action = engine.handle_backspace();
    match action {
        KeyAction::Replace {
            delete_chars,
            delete_bytes,
            text,
        } => {
            assert_eq!(delete_chars, 1);
            assert_eq!(delete_bytes, 'n'.len_utf8());
            assert_eq!(text, "");
            buffer.pop();
        }
        _ => panic!("Expected Replace on backspace"),
    }
    assert_eq!(buffer, "toá");

    // Backspace: removes 'á' (2 bytes in UTF-8)
    let action = engine.handle_backspace();
    match action {
        KeyAction::Replace {
            delete_chars,
            delete_bytes,
            text,
        } => {
            assert_eq!(delete_chars, 1);
            assert_eq!(delete_bytes, 'á'.len_utf8());
            assert_eq!(delete_bytes, 2);
            assert_eq!(text, "");
            buffer.pop();
        }
        _ => panic!("Expected Replace on backspace"),
    }
    assert_eq!(buffer, "to");
}

#[test]
fn test_challenge_phrase_and_boundaries() {
    let mut harness = TestHarness::new(TonePlacementStyle::Modern);

    let phrase = "Vieetj Nam ddaats nuowcs hoaff binhf.";
    // Words:
    // "Vieetj " -> "Việt "
    // "Nam " -> "Nam "
    // "ddaats " -> "đất "
    // "nuowcs " -> "nước "
    // "hoaff " -> "hoaf " (undo f)
    // "binhf." -> "bình."
    let out = harness.feed(phrase);
    assert_eq!(out, "Việt Nam đất nước hoaf bình.");
}
