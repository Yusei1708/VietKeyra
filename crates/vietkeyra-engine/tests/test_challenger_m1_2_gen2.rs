//! Challenger 2 (Iteration 2) Comprehensive Empirical Test Suite
//!
//! Validates:
//! 1. Linguistic correctness of standalone 'w' repeat revert (ww -> w, WW -> W, trww -> trw, etc.)
//! 2. Zero regression across horn vowels (uw -> ư, ow -> ơ, uow -> ươ, aw -> ă) and their repeat undo
//! 3. Case preservation (UPPERCASE, Titlecase, lowercase, mixed case)
//! 4. Wayland UTF-8 delete_bytes oracle with byte-level buffer simulation
//! 5. Backspace behavior and history reset invariants
//! 6. Real-world sentences and URL typing resilience

use vietkeyra_engine::{InputMode, KeyAction, TonePlacementStyle, VietKeyraEngine};

/// Dual-buffer oracle tracking both Unicode character representation
/// and Wayland UTF-8 byte slices simultaneously.
struct WaylandSimHarness {
    engine: VietKeyraEngine,
    char_buffer: String,
    byte_buffer: Vec<u8>,
}

impl WaylandSimHarness {
    fn new(style: TonePlacementStyle) -> Self {
        Self {
            engine: VietKeyraEngine::new(style),
            char_buffer: String::new(),
            byte_buffer: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.engine.reset_word();
        self.char_buffer.clear();
        self.byte_buffer.clear();
    }

    fn feed(&mut self, input: &str) -> String {
        self.reset();
        for ch in input.chars() {
            let action = self.engine.process_char(ch);
            self.apply_action(action, ch);
        }
        self.verify_nfc_and_buffers();
        self.char_buffer.clone()
    }

    fn apply_action(&mut self, action: KeyAction, input_ch: char) {
        match action {
            KeyAction::PassThrough => {
                self.char_buffer.push(input_ch);
                self.byte_buffer
                    .extend_from_slice(input_ch.encode_utf8(&mut [0; 4]).as_bytes());
            }
            KeyAction::Commit { text } => {
                self.char_buffer.push_str(&text);
                self.byte_buffer.extend_from_slice(text.as_bytes());
            }
            KeyAction::Replace {
                delete_chars,
                delete_bytes,
                text,
            } => {
                // 1. Verify delete_chars matches char_buffer
                let total_chars = self.char_buffer.chars().count();
                assert!(
                    delete_chars <= total_chars,
                    "CRITICAL: delete_chars ({}) exceeds char buffer length ({}) for input '{}'",
                    delete_chars,
                    total_chars,
                    input_ch
                );

                // 2. Calculate actual UTF-8 bytes to delete from char_buffer
                let mut actual_delete_bytes = 0;
                let mut counted_chars = 0;
                for c in self.char_buffer.chars().rev() {
                    if counted_chars < delete_chars {
                        counted_chars += 1;
                        actual_delete_bytes += c.len_utf8();
                    } else {
                        break;
                    }
                }

                // 3. EMPIRICAL ORACLE: delete_bytes MUST exactly match actual UTF-8 bytes
                assert_eq!(
                    delete_bytes, actual_delete_bytes,
                    "CRITICAL WAYLAND BYTE CONTRACT VIOLATION: delete_bytes ({}) != actual_delete_bytes ({}) for key '{}'",
                    delete_bytes, actual_delete_bytes, input_ch
                );

                // 4. Update char buffer
                for _ in 0..delete_chars {
                    self.char_buffer.pop();
                }
                self.char_buffer.push_str(&text);

                // 5. Update Wayland byte buffer using delete_bytes strictly
                assert!(
                    delete_bytes <= self.byte_buffer.len(),
                    "CRITICAL: delete_bytes ({}) exceeds byte buffer length ({})",
                    delete_bytes,
                    self.byte_buffer.len()
                );
                let new_len = self.byte_buffer.len() - delete_bytes;
                self.byte_buffer.truncate(new_len);
                self.byte_buffer.extend_from_slice(text.as_bytes());

                // 6. Dual-buffer synchronization check: byte_buffer MUST equal char_buffer as UTF-8
                let byte_as_str = std::str::from_utf8(&self.byte_buffer).expect(
                    "CRITICAL: byte_buffer is corrupted invalid UTF-8 after deletion/replacement",
                );
                assert_eq!(
                    byte_as_str, self.char_buffer,
                    "CRITICAL DESYNCHRONIZATION: char_buffer and byte_buffer diverged! char='{}', byte='{}'",
                    self.char_buffer, byte_as_str
                );
            }
        }
    }

    fn backspace(&mut self) -> KeyAction {
        let action = self.engine.handle_backspace();
        match &action {
            KeyAction::PassThrough => {}
            KeyAction::Commit { text } => {
                self.char_buffer.push_str(text);
                self.byte_buffer.extend_from_slice(text.as_bytes());
            }
            KeyAction::Replace {
                delete_chars,
                delete_bytes,
                text,
            } => {
                assert_eq!(
                    *delete_chars, 1,
                    "Backspace must delete exactly 1 character"
                );
                assert!(
                    *delete_bytes <= self.byte_buffer.len(),
                    "Backspace delete_bytes exceeds byte buffer length"
                );
                let mut actual_bytes = 0;
                if let Some(last_c) = self.char_buffer.chars().last() {
                    actual_bytes = last_c.len_utf8();
                }
                assert_eq!(
                    *delete_bytes, actual_bytes,
                    "Backspace delete_bytes ({}) != actual last char UTF-8 len ({})",
                    delete_bytes, actual_bytes
                );

                self.char_buffer.pop();
                self.char_buffer.push_str(text);

                let new_len = self.byte_buffer.len() - delete_bytes;
                self.byte_buffer.truncate(new_len);
                self.byte_buffer.extend_from_slice(text.as_bytes());

                let byte_as_str = std::str::from_utf8(&self.byte_buffer)
                    .expect("byte_buffer invalid UTF-8 after backspace");
                assert_eq!(byte_as_str, self.char_buffer);
            }
        }
        action
    }

    fn verify_nfc_and_buffers(&self) {
        assert!(
            !self
                .char_buffer
                .chars()
                .any(|c| ('\u{0300}'..='\u{036F}').contains(&c)),
            "Output contains decomposed combining characters: '{}'",
            self.char_buffer
        );
        let byte_as_str = std::str::from_utf8(&self.byte_buffer).unwrap();
        assert_eq!(byte_as_str, self.char_buffer);
    }
}

#[test]
fn test_empirical_standalone_w_revert_all_variants() {
    let mut harness = WaylandSimHarness::new(TonePlacementStyle::Modern);

    let test_cases = [
        // 1. Isolated standalone 'w'
        ("w", "ư"),
        ("ww", "w"),
        ("www", "ww"),
        ("wwww", "www"),
        ("wwwww", "wwww"),
        // 2. Uppercase standalone 'W'
        ("W", "Ư"),
        ("WW", "W"),
        ("WWW", "WW"),
        ("WWWW", "WWW"),
        // 3. Mixed case standalone
        ("wW", "W"),
        ("Ww", "w"),
        // 4. Standalone 'w' after onsets without vowels
        ("trw", "trư"),
        ("trww", "trw"),
        ("trwww", "trww"),
        ("thw", "thư"),
        ("thww", "thw"),
        ("chw", "chư"),
        ("chww", "chw"),
        ("ngw", "ngư"),
        ("ngww", "ngw"),
        ("nghw", "nghư"),
        ("nghww", "nghw"),
        ("phw", "phư"),
        ("phww", "phw"),
        ("khw", "khư"),
        ("khww", "khw"),
        ("nhw", "như"),
        ("nhww", "nhw"),
        ("ghw", "ghư"),
        ("ghww", "ghw"),
        // Single consonant onsets
        ("bw", "bư"),
        ("bww", "bw"),
        ("cw", "cư"),
        ("cww", "cw"),
        ("dw", "dư"),
        ("dww", "dw"),
        ("ddw", "đư"),
        ("ddww", "đw"),
        ("gw", "gư"),
        ("gww", "gw"),
        ("hw", "hư"),
        ("hww", "hw"),
        ("kw", "kư"),
        ("kww", "kw"),
        ("lw", "lư"),
        ("lww", "lw"),
        ("mw", "mư"),
        ("mww", "mw"),
        ("nw", "nư"),
        ("nww", "nw"),
        ("rw", "rư"),
        ("rww", "rw"),
        ("sw", "sư"),
        ("sww", "sw"),
        ("tw", "tư"),
        ("tww", "tw"),
        ("vw", "vư"),
        ("vww", "vw"),
        ("xw", "xư"),
        ("xww", "xw"),
        // Uppercase onsets
        ("Trw", "Trư"),
        ("Trww", "Trw"),
        ("TRW", "TRƯ"),
        ("TRWW", "TRW"),
        ("DDW", "ĐƯ"),
        ("DDWW", "ĐW"),
        ("Ddw", "Đư"),
        ("Ddww", "Đw"),
    ];

    for (input, expected) in test_cases {
        let actual = harness.feed(input);
        assert_eq!(
            actual, expected,
            "Failed standalone w revert test for '{}': got '{}', expected '{}'",
            input, actual, expected
        );
    }
}

#[test]
fn test_empirical_horn_vowels_zero_regression() {
    let mut harness = WaylandSimHarness::new(TonePlacementStyle::Modern);

    let horn_cases = [
        // uw -> ư, uww -> uw
        ("uw", "ư"),
        ("uww", "uw"),
        ("uwww", "uww"),
        ("UW", "Ư"),
        ("UWW", "UW"),
        ("tuw", "tư"),
        ("tuww", "tuw"),
        // ow -> ơ, oww -> ow
        ("ow", "ơ"),
        ("oww", "ow"),
        ("owww", "oww"),
        ("OW", "Ơ"),
        ("OWW", "OW"),
        ("mow", "mơ"),
        ("moww", "mow"),
        // uow -> ươ, uoww -> uow
        ("uow", "ươ"),
        ("uoww", "uow"),
        ("uowww", "uoww"),
        ("UOW", "ƯƠ"),
        ("UOWW", "UOW"),
        ("nuowcs", "nước"),
        ("dduowfng", "đường"),
        ("nguowfi", "người"),
        ("ruowuj", "rượu"),
        // aw -> ă, aww -> aw
        ("aw", "ă"),
        ("aww", "aw"),
        ("awww", "aww"),
        ("AW", "Ă"),
        ("AWW", "AW"),
        ("cawn", "căn"),
        ("cawwn", "cawn"),
        // Full horn tone matrix
        ("tuwf", "từ"),
        ("tuws", "tứ"),
        ("tuwr", "tử"),
        ("tuwx", "tữ"),
        ("tuwj", "tự"),
        ("mowf", "mờ"),
        ("mows", "mớ"),
        ("mowr", "mở"),
        ("mowx", "mỡ"),
        ("mowj", "mợ"),
        ("awn", "ăn"),
        ("tawns", "tắn"),
        ("lawngj", "lặng"),
        ("nawm", "năm"),
    ];

    for (input, expected) in horn_cases {
        let actual = harness.feed(input);
        assert_eq!(
            actual, expected,
            "Failed horn vowel test for '{}': got '{}', expected '{}'",
            input, actual, expected
        );
    }
}

#[test]
fn test_empirical_modern_vs_traditional_contrast() {
    let mut modern_harness = WaylandSimHarness::new(TonePlacementStyle::Modern);
    let mut trad_harness = WaylandSimHarness::new(TonePlacementStyle::Traditional);

    let contrasts = [
        ("hoaf", "hoà", "hòa"),
        ("khoer", "khoẻ", "khỏe"),
        ("thuyr", "thuỷ", "thủy"),
        ("nguyj", "nguỵ", "ngụy"),
        ("luyx", "luỹ", "lũy"),
        ("tuyf", "tuỳ", "tùy"),
        ("xoer", "xoẻ", "xỏe"),
        ("loer", "loẻ", "lỏe"),
        ("thuys", "thuý", "thúy"),
        ("toas", "toá", "tóa"),
    ];

    for (input, exp_modern, exp_trad) in contrasts {
        let act_modern = modern_harness.feed(input);
        let act_trad = trad_harness.feed(input);
        assert_eq!(
            act_modern, exp_modern,
            "Modern tone mismatch for '{}': got '{}', expected '{}'",
            input, act_modern, exp_modern
        );
        assert_eq!(
            act_trad, exp_trad,
            "Traditional tone mismatch for '{}': got '{}', expected '{}'",
            input, act_trad, exp_trad
        );
    }
}

#[test]
fn test_empirical_case_preservation_comprehensive() {
    let mut harness = WaylandSimHarness::new(TonePlacementStyle::Modern);

    let casing_cases = [
        ("VIEETJ", "VIỆT"),
        ("NAM", "NAM"),
        ("DDOONGF", "ĐỒNG"),
        ("NAI", "NAI"),
        ("DDUOWFNG", "ĐƯỜNG"),
        ("XAX", "XÃ"),
        ("QUYEETS", "QUYẾT"),
        ("TAAM", "TÂM"),
        ("GIEENGS", "GIẾNG"),
        ("NUOWCS", "NƯỚC"),
        ("THUYR", "THUỶ"),
        ("HOAF", "HOÀ"),
        ("BINHF", "BÌNH"),
        // Titlecase
        ("Vieetj", "Việt"),
        ("Ddoongf", "Đồng"),
        ("Dduowfng", "Đường"),
        ("Quyeets", "Quyết"),
        ("Gieengs", "Giếng"),
        ("Thuyr", "Thuỷ"),
        ("Hoaf", "Hoà"),
        // D-stroke casing
        ("dd", "đ"),
        ("DD", "Đ"),
        ("Dd", "Đ"),
        ("dD", "Đ"),
    ];

    for (input, expected) in casing_cases {
        let actual = harness.feed(input);
        assert_eq!(
            actual, expected,
            "Casing preservation failed for '{}': got '{}', expected '{}'",
            input, actual, expected
        );
    }
}

#[test]
fn test_empirical_backspace_state_resets_cleanly() {
    let mut harness = WaylandSimHarness::new(TonePlacementStyle::Modern);

    // 1. Type "ww" -> "w", then backspace to empty, then type "w" -> should produce "ư"
    harness.feed("ww");
    assert_eq!(harness.char_buffer, "w");
    assert!(!harness.engine.is_empty());
    assert_eq!(harness.engine.history_len(), 2);

    let action = harness.backspace();
    assert_eq!(
        action,
        KeyAction::Replace {
            delete_chars: 1,
            delete_bytes: 1,
            text: String::new()
        }
    );
    assert_eq!(harness.char_buffer, "");
    assert!(harness.engine.is_empty());
    assert_eq!(harness.engine.history_len(), 0);

    // Now type 'w' -> should be standalone 'w', producing 'ư'
    let act = harness.engine.process_char('w');
    harness.apply_action(act, 'w');
    assert_eq!(harness.char_buffer, "ư");

    // 2. Type "trww" -> "trw", backspace 3 times -> empty -> history must be 0
    harness.feed("trww");
    assert_eq!(harness.char_buffer, "trw");
    harness.backspace(); // pops 'w'
    assert_eq!(harness.char_buffer, "tr");
    harness.backspace(); // pops 'r'
    assert_eq!(harness.char_buffer, "t");
    harness.backspace(); // pops 't'
    assert_eq!(harness.char_buffer, "");
    assert!(harness.engine.is_empty());
    assert_eq!(harness.engine.history_len(), 0);

    // Consecutive backspaces when already empty should return PassThrough
    let empty_act = harness.backspace();
    assert_eq!(empty_act, KeyAction::PassThrough);
    assert!(harness.engine.is_empty());
    assert_eq!(harness.engine.history_len(), 0);
}

#[test]
fn test_empirical_sentences_and_urls() {
    let mut harness = WaylandSimHarness::new(TonePlacementStyle::Modern);

    // Typing sentence with standalone w and horn vowels
    let sentence1 = "Trowif muwa uowts aos em tooi, dduowfng xa.";
    assert_eq!(harness.feed(sentence1), "Trời mưa ướt áo em tôi, đường xa.");

    // Standalone w in sentence
    let sentence2 = "Tooi ww xe.";
    assert_eq!(harness.feed(sentence2), "Tôi w xe.");

    // Non-telex URL in VI mode:
    let url = "wwww.github.com";
    assert_eq!(harness.feed(url), "www.github.com");

    // English mode switches for English domain names
    harness.engine.set_mode(InputMode::English);
    assert_eq!(harness.feed("www.rust-lang.org"), "www.rust-lang.org");
    assert_eq!(harness.feed("www.google.com"), "www.google.com");
    harness.engine.set_mode(InputMode::Vietnamese);
}
