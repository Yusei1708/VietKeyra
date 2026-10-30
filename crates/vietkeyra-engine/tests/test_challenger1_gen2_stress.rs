//! Empirical Stress Test Suite for vietkeyra-engine (Milestone 1 Iteration 2)
//!
//! Evaluates standalone 'w' repeat reversions, Backspace empty buffer sequences,
//! zero history bleed, 100,000+ keystroke memory/latency stability, and fuzzing invariants.

use std::fs;
use std::time::Instant;
use vietkeyra_engine::{nfc::decompose_vowel, KeyAction, TonePlacementStyle, VietKeyraEngine};

/// High-fidelity editor simulation tracking character count and exact UTF-8 byte offsets.
struct EditorSimulator {
    engine: VietKeyraEngine,
    buffer: String,
}

impl EditorSimulator {
    fn new(style: TonePlacementStyle) -> Self {
        Self {
            engine: VietKeyraEngine::new(style),
            buffer: String::new(),
        }
    }

    fn type_char(&mut self, ch: char) {
        match self.engine.process_char(ch) {
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
                let char_count = self.buffer.chars().count();
                assert!(
                    char_count >= delete_chars,
                    "CRITICAL: delete_chars ({}) exceeds buffer char count ({}) for key '{}', buffer: {:?}",
                    delete_chars,
                    char_count,
                    ch,
                    self.buffer
                );

                let keep_chars = char_count - delete_chars;
                let keep_byte_idx = self
                    .buffer
                    .char_indices()
                    .nth(keep_chars)
                    .map(|(idx, _)| idx)
                    .unwrap_or(self.buffer.len());

                let deleted_slice = &self.buffer[keep_byte_idx..];
                assert_eq!(
                    deleted_slice.len(),
                    delete_bytes,
                    "CRITICAL UTF-8 BYTE DESYNC: expected delete_bytes {}, but deleted slice {:?} has byte len {} for key '{}'",
                    delete_bytes,
                    deleted_slice,
                    deleted_slice.len(),
                    ch
                );

                self.buffer.truncate(keep_byte_idx);
                self.buffer.push_str(&text);
            }
        }
    }

    fn backspace(&mut self) {
        match self.engine.handle_backspace() {
            KeyAction::PassThrough => {
                self.buffer.pop();
            }
            KeyAction::Commit { text } => {
                self.buffer.push_str(&text);
            }
            KeyAction::Replace {
                delete_chars,
                delete_bytes,
                text,
            } => {
                let char_count = self.buffer.chars().count();
                assert!(
                    char_count >= delete_chars,
                    "CRITICAL: delete_chars ({}) exceeds buffer char count ({}) on backspace, buffer: {:?}",
                    delete_chars,
                    char_count,
                    self.buffer
                );

                let keep_chars = char_count - delete_chars;
                let keep_byte_idx = self
                    .buffer
                    .char_indices()
                    .nth(keep_chars)
                    .map(|(idx, _)| idx)
                    .unwrap_or(self.buffer.len());

                let deleted_slice = &self.buffer[keep_byte_idx..];
                assert_eq!(
                    deleted_slice.len(),
                    delete_bytes,
                    "CRITICAL UTF-8 BYTE DESYNC ON BACKSPACE: expected delete_bytes {}, got {} for slice {:?}",
                    delete_bytes,
                    deleted_slice.len(),
                    deleted_slice
                );

                self.buffer.truncate(keep_byte_idx);
                self.buffer.push_str(&text);
            }
        }
    }

    fn type_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.type_char(ch);
        }
    }

    fn buffer(&self) -> &str {
        &self.buffer
    }

    fn reset(&mut self) {
        self.engine.reset_word();
        self.buffer.clear();
    }
}

fn assert_nfc_valid(text: &str) {
    for ch in text.chars() {
        if let Some((base, modifier, tone, case)) = decompose_vowel(ch) {
            let composed = vietkeyra_engine::nfc::compose_vowel(base, modifier, tone, case);
            assert_eq!(
                composed,
                Some(ch),
                "Char '{}' is not valid canonical NFC Unicode!",
                ch
            );
        }
    }
}

// ============================================================================
// 1. STANDALONE 'W' REPEAT REVERSION STRESS TESTS
// ============================================================================

#[test]
fn test_stress_standalone_w_repeat_chains() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);

    // Basic w repeat chains:
    // w -> ư
    // ww -> w
    // www -> ww
    // wwww -> www
    // wwwww -> wwww
    // wwwwww -> wwwww
    let w_chains = [
        ("w", "ư"),
        ("ww", "w"),
        ("www", "ww"),
        ("wwww", "www"),
        ("wwwww", "wwww"),
        ("wwwwww", "wwwww"),
    ];

    for (input, expected) in w_chains {
        sim.reset();
        sim.type_str(input);
        assert_nfc_valid(sim.buffer());
        assert_eq!(
            sim.buffer(),
            expected,
            "Failed standalone w chain for input '{}'",
            input
        );
    }
}

#[test]
fn test_stress_standalone_w_uppercase_and_mixed_case() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);

    let upper_chains = [
        ("W", "Ư"),
        ("WW", "W"),
        ("WWW", "WW"),
        ("WWWW", "WWW"),
        ("WWWWW", "WWWW"),
        // Mixed casing
        ("wW", "W"),
        ("Ww", "w"),
        ("wWw", "Ww"),
        ("WwW", "wW"),
        ("wWW", "WW"),
        ("WWw", "Ww"),
    ];

    for (input, expected) in upper_chains {
        sim.reset();
        sim.type_str(input);
        assert_nfc_valid(sim.buffer());
        assert_eq!(
            sim.buffer(),
            expected,
            "Failed upper/mixed w chain for input '{}'",
            input
        );
    }
}

#[test]
fn test_stress_consonant_onset_standalone_w_chains() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);

    // tr + w repetitions:
    // trw -> trư
    // trww -> trw
    // trwww -> trww
    // trwwww -> trwww
    let tr_chains = [
        ("trw", "trư"),
        ("trww", "trw"),
        ("trwww", "trww"),
        ("trwwww", "trwww"),
        ("Trw", "Trư"),
        ("Trww", "Trw"),
        ("Trwww", "Trww"),
        ("TRW", "TRƯ"),
        ("TRWW", "TRW"),
        ("TRWWW", "TRWW"),
    ];

    for (input, expected) in tr_chains {
        sim.reset();
        sim.type_str(input);
        assert_nfc_valid(sim.buffer());
        assert_eq!(
            sim.buffer(),
            expected,
            "Failed consonant onset w chain for input '{}'",
            input
        );
    }

    // Diverse Vietnamese onsets followed by w and repeat w:
    let onset_cases = [
        ("thw", "thư", "thww", "thw", "thwww", "thww"),
        ("khw", "khư", "khww", "khw", "khwww", "khww"),
        ("chw", "chư", "chww", "chw", "chwww", "chww"),
        ("nhw", "như", "nhww", "nhw", "nhwww", "nhww"),
        ("ngw", "ngư", "ngww", "ngw", "ngwww", "ngww"),
        ("phw", "phư", "phww", "phw", "phwww", "phww"),
        ("bw", "bư", "bww", "bw", "bwww", "bww"),
        ("cw", "cư", "cww", "cw", "cwww", "cww"),
        ("dw", "dư", "dww", "dw", "dwww", "dww"),
        ("gw", "gư", "gww", "gw", "gwww", "gww"),
        ("hw", "hư", "hww", "hw", "hwww", "hww"),
        ("lw", "lư", "lww", "lw", "lwww", "lww"),
        ("mw", "mư", "mww", "mw", "mwww", "mww"),
        ("nw", "nư", "nww", "nw", "nwww", "nww"),
        ("rw", "rư", "rww", "rw", "rwww", "rww"),
        ("sw", "sư", "sww", "sw", "swwww", "swww"),
        ("tw", "tư", "tww", "tw", "twww", "tww"),
        ("vw", "vư", "vww", "vw", "vwww", "vww"),
        ("xw", "xư", "xww", "xw", "xwww", "xww"),
    ];

    for (in1, exp1, in2, exp2, in3, exp3) in onset_cases {
        sim.reset();
        sim.type_str(in1);
        assert_eq!(sim.buffer(), exp1);

        sim.reset();
        sim.type_str(in2);
        assert_eq!(sim.buffer(), exp2);

        sim.reset();
        sim.type_str(in3);
        assert_eq!(sim.buffer(), exp3);
    }
}

#[test]
fn test_stress_d_stroke_and_w_combinations() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);

    // dd -> đ, ddw -> đư, ddww -> đw, ddwww -> đww
    let d_cases = [
        ("ddw", "đư"),
        ("ddww", "đw"),
        ("ddwww", "đww"),
        ("ddwwww", "đwww"),
        ("DDW", "ĐƯ"),
        ("DDWW", "ĐW"),
        ("DDWWW", "ĐWW"),
    ];

    for (input, expected) in d_cases {
        sim.reset();
        sim.type_str(input);
        assert_nfc_valid(sim.buffer());
        assert_eq!(
            sim.buffer(),
            expected,
            "Failed d-stroke + w chain for input '{}'",
            input
        );
    }
}

#[test]
fn test_stress_vowel_horn_w_repeat_chains() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);

    // a + w: aw -> ă, aww -> aw, awww -> aww, awwww -> awww
    // u + w: uw -> ư, uww -> uw, uwww -> uww, uwwww -> uwww
    // o + w: ow -> ơ, oww -> ow, owww -> oww, owwww -> owww
    // uo + w: uow -> ươ, uoww -> uow, uowww -> uoww
    let vowel_w_cases = [
        ("aw", "ă"),
        ("aww", "aw"),
        ("awww", "aww"),
        ("awwww", "awww"),
        ("uw", "ư"),
        ("uww", "uw"),
        ("uwww", "uww"),
        ("uwwww", "uwww"),
        ("ow", "ơ"),
        ("oww", "ow"),
        ("owww", "oww"),
        ("owwww", "owww"),
        ("uow", "ươ"),
        ("uoww", "uow"),
        ("uowww", "uoww"),
        ("uowwww", "uowww"),
    ];

    for (input, expected) in vowel_w_cases {
        sim.reset();
        sim.type_str(input);
        assert_nfc_valid(sim.buffer());
        assert_eq!(
            sim.buffer(),
            expected,
            "Failed vowel horn w chain for input '{}'",
            input
        );
    }
}

// ============================================================================
// 2. BACKSPACE SEQUENCES & ZERO HISTORY BLEED STRESS TESTS
// ============================================================================

#[test]
fn test_stress_backspace_to_empty_and_zero_history_bleed() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);

    // Words with multi-keystroke compositions and tones
    let words = [
        "toans",    // toán (5 strokes -> 4 chars)
        "nguyeexn", // nguyễn (8 strokes -> 6 chars)
        "thuyeefn", // thuyền (8 strokes -> 6 chars)
        "dduowfng", // đường (8 strokes -> 5 chars)
        "phwowng",  // phương (7 strokes -> 6 chars)
        "khoees",   // khoé (6 strokes -> 4 chars)
        "vieejt",   // việt (6 strokes -> 4 chars)
        "tieesng",  // tiếng (7 strokes -> 5 chars)
    ];

    let post_keys = ['s', 'f', 'r', 'x', 'j', 'w', 'a', 'e', 'o', 'd'];

    for word in words {
        for next_key in post_keys {
            sim.reset();
            sim.type_str(word);
            assert_nfc_valid(sim.buffer());

            // Backspace exactly until empty
            let len = sim.buffer().chars().count();
            for _ in 0..len {
                sim.backspace();
            }
            assert_eq!(
                sim.buffer(),
                "",
                "Buffer must be empty after full backspace"
            );
            assert!(
                sim.engine.is_empty(),
                "Engine must report is_empty() after full backspace"
            );
            assert_eq!(
                sim.engine.history_len(),
                0,
                "Engine history must be strictly 0 after empty buffer"
            );

            // Type the post key: it must act as a brand-new onset/letter, NOT a diacritic modifier!
            sim.type_char(next_key);
            let expected = if next_key == 'w' {
                "ư"
            } else {
                &next_key.to_string()
            };
            assert_eq!(
                sim.buffer(),
                expected,
                "History bleed detected! After backspacing '{}' to empty, key '{}' produced '{}' instead of '{}'",
                word,
                next_key,
                sim.buffer(),
                expected
            );
        }
    }
}

#[test]
fn test_stress_excessive_backspaces_on_empty_buffer() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);

    // 100 backspaces on clean engine
    for _ in 0..100 {
        sim.backspace();
        assert_eq!(sim.buffer(), "");
        assert_eq!(sim.engine.history_len(), 0);
    }

    // Now type a complex word
    sim.type_str("ddieefu");
    assert_eq!(sim.buffer(), "điều");

    // 50 excessive backspaces
    for _ in 0..50 {
        sim.backspace();
    }
    assert_eq!(sim.buffer(), "");
    assert_eq!(sim.engine.history_len(), 0);

    // Type 'w': should be 'ư'
    sim.type_char('w');
    assert_eq!(sim.buffer(), "ư");
    assert_eq!(sim.engine.history_len(), 1);

    // Backspace once
    sim.backspace();
    assert_eq!(sim.buffer(), "");
    assert_eq!(sim.engine.history_len(), 0);

    // Type 'w' again: should be 'ư' (NOT 'w' from a phantom previous 'ư')
    sim.type_char('w');
    assert_eq!(sim.buffer(), "ư");
}

#[test]
fn test_stress_interleaved_backspace_and_word_reconstruction() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);

    // Partial backspaces and resumes
    sim.type_str("hoaf"); // hoà
    assert_eq!(sim.buffer(), "hoà");

    sim.backspace(); // pops 'à' -> 'ho'
    assert_eq!(sim.buffer(), "ho");

    sim.type_str("ng"); // hong
    assert_eq!(sim.buffer(), "hong");

    sim.type_char('f'); // hòng
    assert_eq!(sim.buffer(), "hòng");

    // Backspace 2 times
    sim.backspace(); // pops 'g'
    sim.backspace(); // pops 'n' -> 'hò'
    assert_eq!(sim.buffer(), "hò");

    sim.type_str("a"); // hò + a -> hoà
    assert_eq!(sim.buffer(), "hoà");

    sim.type_char('n'); // hoàn
    assert_eq!(sim.buffer(), "hoàn");

    sim.type_char('g'); // hoàng
    assert_eq!(sim.buffer(), "hoàng");
}

#[test]
fn test_stress_single_char_rapid_backspace_cycles() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);

    for ch in ['a', 'w', 'd', 's', 'e', 'o'] {
        for _ in 0..50 {
            sim.type_char(ch);
            assert!(!sim.buffer().is_empty());
            sim.backspace();
            assert_eq!(sim.buffer(), "");
            assert_eq!(sim.engine.history_len(), 0);
        }
    }
}

// ============================================================================
// 3. 100,000+ OPERATION LATENCY AND MEMORY STABILITY
// ============================================================================

fn get_process_rss_kb() -> usize {
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<usize>() {
                        return kb;
                    }
                }
            }
        }
    }
    0
}

#[test]
#[allow(clippy::explicit_counter_loop)]
fn test_stress_100k_operations_stability_and_latency() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let sample_stream = [
        "tieng", "s", " ", "viet", "j", " ", "nam", " ", "doan", "f", " ", "ket", "s", " ",
        "thanh", "f", " ", "cong", " ", "ww", " ", "www", " ", "trww", " ", "uow", " ", "a", "s",
        "s", " ", "dd", "d", " ", "toans", " ",
    ];

    let start_rss = get_process_rss_kb();
    let total_operations = 120_000;
    let mut total_duration = std::time::Duration::ZERO;
    let mut max_op_nanos = 0u128;

    let overall_timer = Instant::now();

    for op in 0..total_operations {
        let chunk = sample_stream[op % sample_stream.len()];

        for ch in chunk.chars() {
            let t0 = Instant::now();
            let action = engine.process_char(ch);
            let elapsed = t0.elapsed();
            total_duration += elapsed;
            let nanos = elapsed.as_nanos();
            if nanos > max_op_nanos {
                max_op_nanos = nanos;
            }

            // Invariant check on action
            match action {
                KeyAction::PassThrough => {}
                KeyAction::Commit { .. } => {}
                KeyAction::Replace {
                    delete_chars,
                    delete_bytes,
                    ref text,
                } => {
                    assert_nfc_valid(text);
                    if delete_chars > 0 {
                        assert!(delete_bytes >= delete_chars);
                    }
                }
            }
        }

        // Interleave backspaces every 50 operations
        if op % 50 == 0 {
            for _ in 0..5 {
                let t0 = Instant::now();
                let _ = engine.handle_backspace();
                let elapsed = t0.elapsed();
                total_duration += elapsed;
            }
        }

        // Interleave mode switches every 500 operations
        if op % 500 == 0 {
            let _ = engine.toggle_mode();
        }
    }

    let end_rss = get_process_rss_kb();
    let total_elapsed = overall_timer.elapsed();
    let avg_op_nanos = total_duration.as_nanos() / (total_operations as u128);

    println!(
        "120,000 Operations Stress Test Results:\n\
         - Total time: {:?}\n\
         - Average latency per keystroke: {} ns\n\
         - Max latency observed: {} ns ({} µs)\n\
         - Initial RSS: {} KB\n\
         - Final RSS: {} KB\n\
         - Memory Delta: {} KB",
        total_elapsed,
        avg_op_nanos,
        max_op_nanos,
        max_op_nanos / 1000,
        start_rss,
        end_rss,
        end_rss.saturating_sub(start_rss)
    );

    // Latency assertion: average latency must be well below 2,000 ns (2 µs), achieving sub-microsecond
    assert!(
        avg_op_nanos < 2000,
        "Average keystroke latency is too slow: {} ns",
        avg_op_nanos
    );

    // Memory assertion: zero heap leak (delta must be < 512 KB over 120k ops)
    let mem_delta = end_rss.saturating_sub(start_rss);
    assert!(
        mem_delta < 512,
        "Memory leak detected! RSS grew by {} KB across 120,000 operations",
        mem_delta
    );
}

// ============================================================================
// 4. ADVERSARIAL FUZZING WITH STRICT INVARIANT VERIFICATION
// ============================================================================

#[test]
fn test_stress_adversarial_fuzzing_invariants() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);

    // LCG PRNG for determinism
    let mut lcg_state: u64 = 0xDEADBEEFCAFE;
    let mut rand_u32 = || -> u32 {
        lcg_state = lcg_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (lcg_state >> 32) as u32
    };

    let alphabet = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'D', 'E', 'O', 'U', 'W', 'S', 'F', 'R',
        'X', 'J', ' ', '.', ',', ';', '!', '?', '-', '0', '1', '2',
    ];

    let fuzz_steps = 30_000;

    for i in 0..fuzz_steps {
        let action_choice = rand_u32() % 100;

        if action_choice < 15 {
            // 15% backspace
            sim.backspace();
        } else if action_choice < 20 {
            // 5% multiple backspaces
            let count = (rand_u32() % 8) + 1;
            for _ in 0..count {
                sim.backspace();
            }
        } else if action_choice < 23 {
            // 3% mode toggle
            sim.engine.toggle_mode();
        } else {
            // 77% keystroke
            let char_idx = (rand_u32() as usize) % alphabet.len();
            let ch = alphabet[char_idx];
            sim.type_char(ch);
        }

        // --- Invariant 1: Buffer must always be canonical NFC ---
        assert_nfc_valid(sim.buffer());

        // --- Invariant 2: Engine emptiness consistency ---
        assert_eq!(
            sim.engine.is_empty(),
            sim.engine.current_word().is_empty(),
            "Invariant violation at step {}: is_empty() mismatch!",
            i
        );

        // --- Invariant 3: History length must be 0 if engine is empty ---
        if sim.engine.is_empty() {
            assert_eq!(
                sim.engine.history_len(),
                0,
                "Invariant violation at step {}: history_len ({}) != 0 on empty engine!",
                i,
                sim.engine.history_len()
            );
        }
    }
}
