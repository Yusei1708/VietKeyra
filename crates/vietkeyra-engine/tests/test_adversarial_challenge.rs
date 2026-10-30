//! Adversarial Challenge Test Harness for vietkeyra-engine
//!
//! Stress-tests boundary conditions, pathological inputs, repeat-key undo cycles,
//! mixed-tone storms, Unicode NFC invariants, zero memory leaks, and Wayland UTF-8 byte synchronization.

use std::fs;
use std::time::Instant;
use vietkeyra_engine::{
    nfc::{decompose_vowel, BaseVowel, Case, Modifier, Tone},
    InputMode, KeyAction, TonePlacementStyle, VietKeyraEngine,
};

/// A high-fidelity editor buffer simulation tracking both character count and UTF-8 byte offsets.
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
                    "delete_chars ({}) > buffer char count ({}) for key '{}', buffer: {:?}",
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
                    "delete_bytes mismatch for key '{}'! Expected {}, but deleted slice {:?} has len {}",
                    ch,
                    delete_bytes,
                    deleted_slice,
                    deleted_slice.len()
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
                    "delete_chars ({}) > buffer char count ({}) on backspace, buffer: {:?}",
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
                    "delete_bytes mismatch on backspace! Expected {}, got {}",
                    delete_bytes,
                    deleted_slice.len()
                );

                self.buffer.truncate(keep_byte_idx);
                self.buffer.push_str(&text);
            }
        }
    }

    fn type_str(&mut self, input: &str) {
        for ch in input.chars() {
            self.type_char(ch);
        }
    }

    fn buffer(&self) -> &str {
        &self.buffer
    }
}

/// Asserts that a string contains NO decomposed combining diacritical marks (U+0300..=U+036F).
fn assert_nfc_invariant(text: &str) {
    for ch in text.chars() {
        assert!(
            !('\u{0300}'..='\u{036F}').contains(&ch),
            "NFC invariant violated! Found combining codepoint U+{:04X} in string {:?}",
            ch as u32,
            text
        );
    }
}

fn get_vm_rss_kb() -> usize {
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
fn test_pathological_repeats_toooansssss() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);
    sim.type_str("toooansssss");
    assert_nfc_invariant(sim.buffer());
    assert!(!sim.buffer().is_empty());
}

#[test]
fn test_pathological_repeats_dddddd() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);
    sim.type_str("dddddd");
    assert_nfc_invariant(sim.buffer());
    // Trace: d -> d, dd -> đ, ddd -> dd, dddd -> dđ, ddddd -> ddd, dddddd -> ddđ
    assert_eq!(sim.buffer(), "ddđ");
}

#[test]
fn test_pathological_repeats_uowwwwww() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);
    sim.type_str("uowwwwww");
    assert_nfc_invariant(sim.buffer());
    assert!(!sim.buffer().is_empty());
}

#[test]
fn test_mixed_tone_storm_asfrxj() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);
    sim.type_str("asfrxj");
    assert_nfc_invariant(sim.buffer());
    assert_eq!(sim.buffer(), "ạ");

    // Reverse tone storm
    sim.type_str(" jxrfsa");
    assert_nfc_invariant(sim.buffer());
}

#[test]
fn test_rapid_alternating_tone_undo() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);
    sim.type_str("toans");
    assert_eq!(sim.buffer(), "toán");
    assert_nfc_invariant(sim.buffer());

    sim.type_char('s');
    assert_eq!(sim.buffer(), "toans");
    assert_nfc_invariant(sim.buffer());
}

#[test]
fn test_interleaved_backspace_and_typing() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);
    sim.type_str("toan");
    assert_eq!(sim.buffer(), "toan");

    sim.type_char('s');
    assert_eq!(sim.buffer(), "toán");

    // Backspace: deletes 'n'
    sim.backspace();
    assert_eq!(sim.buffer(), "toá");

    // Type 'n' again
    sim.type_char('n');
    assert_nfc_invariant(sim.buffer());

    // Backspace all the way to empty
    for _ in 0..10 {
        sim.backspace();
    }
    assert_eq!(sim.buffer(), "");

    // Type again after full deletion
    sim.type_str("vieetj");
    assert_eq!(sim.buffer(), "việt");
    assert_nfc_invariant(sim.buffer());
}

#[test]
fn test_free_tone_placement_mid_word() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);
    // User types tone before coda: 'toasn' -> 'toán'
    sim.type_str("toasn ");
    assert_eq!(sim.buffer(), "toán ");
    assert_nfc_invariant(sim.buffer());

    // User types 'tieesn' -> 'tiến'
    sim.type_str("tieesn ");
    assert_eq!(sim.buffer(), "toán tiến ");
    assert_nfc_invariant(sim.buffer());

    // User types 'cuoosn' -> 'cuốn'
    sim.type_str("cuoosn ");
    assert_eq!(sim.buffer(), "toán tiến cuốn ");
    assert_nfc_invariant(sim.buffer());

    // In Traditional style: 'hoasn' -> 'hoán' (moves tone from o to a due to coda n)
    let mut trad_sim = EditorSimulator::new(TonePlacementStyle::Traditional);
    trad_sim.type_str("hoasn ");
    assert_eq!(trad_sim.buffer(), "hoán ");
    assert_nfc_invariant(trad_sim.buffer());
}

#[test]
fn test_consonant_only_no_accents() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);
    sim.type_str("strs brrf phj ");
    assert_eq!(sim.buffer(), "strs brrf phj ");
    assert_nfc_invariant(sim.buffer());
}

#[test]
fn test_all_144_vowel_nfc_direct_composition() {
    let base_vowels = [
        BaseVowel::A,
        BaseVowel::E,
        BaseVowel::I,
        BaseVowel::O,
        BaseVowel::U,
        BaseVowel::Y,
    ];
    let modifiers = [
        Modifier::None,
        Modifier::Circumflex,
        Modifier::Breve,
        Modifier::Horn,
    ];
    let tones = [
        Tone::None,
        Tone::Sac,
        Tone::Huyen,
        Tone::Hoi,
        Tone::Nga,
        Tone::Nang,
    ];
    let cases = [Case::Lower, Case::Upper];

    let mut count = 0;
    for b in &base_vowels {
        for m in &modifiers {
            for t in &tones {
                for c in &cases {
                    if let Some(ch) = vietkeyra_engine::nfc::compose_vowel(*b, *m, *t, *c) {
                        count += 1;
                        assert_nfc_invariant(&ch.to_string());
                        let (db, dm, dt, dc) =
                            decompose_vowel(ch).expect("every composed vowel must decompose");
                        assert_eq!(db, *b);
                        assert_eq!(dm, *m);
                        assert_eq!(dt, *t);
                        assert_eq!(dc, *c);
                    }
                }
            }
        }
    }
    // 144 total valid vowel variants (72 lower + 72 upper)
    assert_eq!(count, 144);
}

#[test]
fn test_case_preservation_chaos() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);
    sim.type_str("VIEETJ ");
    assert_eq!(sim.buffer(), "VIỆT ");
    assert_nfc_invariant(sim.buffer());

    sim.type_str("DDOONGF ");
    assert_eq!(sim.buffer(), "VIỆT ĐỒNG ");
    assert_nfc_invariant(sim.buffer());

    sim.type_str("Vieetj ");
    assert_eq!(sim.buffer(), "VIỆT ĐỒNG Việt ");
    assert_nfc_invariant(sim.buffer());
}

#[test]
fn test_special_onsets_qu_and_gi() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);
    sim.type_str("quans ");
    sim.type_str("quyeets ");
    sim.type_str("quys ");
    sim.type_str("giaf ");
    sim.type_str("giowf ");
    sim.type_str("ginf ");
    sim.type_str("gif ");
    assert_eq!(sim.buffer(), "quán quyết quý già giờ gìn gì ");
    assert_nfc_invariant(sim.buffer());
}

#[test]
fn test_punctuation_and_mode_switching() {
    let mut sim = EditorSimulator::new(TonePlacementStyle::Modern);
    sim.type_str("toan, hoa! dep? ");
    assert_eq!(sim.buffer(), "toan, hoa! dep? ");

    sim.type_str("vieetj. ");
    assert_eq!(sim.buffer(), "toan, hoa! dep? việt. ");

    sim.engine.set_mode(InputMode::English);
    sim.type_str("vieetj ");
    assert_eq!(sim.buffer(), "toan, hoa! dep? việt. vieetj ");

    sim.engine.set_mode(InputMode::Vietnamese);
    sim.type_str("vieetj ");
    assert_eq!(sim.buffer(), "toan, hoa! dep? việt. vieetj việt ");
}

#[test]
fn test_fuzzing_and_sub_microsecond_benchmark() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let keys = [
        'a', 'a', 'w', 's', 'f', 'r', 'x', 'j', 'd', 'd', 'e', 'e', 'o', 'o', 'w', 'u', 'i', 'n',
        'h', 'g', 't', 'c', 'p', 'm', 'y', ' ', '.', ',', 'q', 'u',
    ];

    let total_keystrokes = 100_000;
    let mut state: u32 = 42;

    let start = Instant::now();
    for _ in 0..total_keystrokes {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        let key = keys[(state as usize) % keys.len()];
        let _action = engine.process_char(key);
    }
    let duration = start.elapsed();
    let nanos_per_keystroke = duration.as_nanos() as f64 / total_keystrokes as f64;
    println!(
        "Benchmark: {} keystrokes executed in {:?}, avg {:.2} ns/keystroke ({:.4} µs)",
        total_keystrokes,
        duration,
        nanos_per_keystroke,
        nanos_per_keystroke / 1000.0
    );

    #[cfg(not(debug_assertions))]
    assert!(
        nanos_per_keystroke < 1000.0,
        "Performance regression! Keystroke latency was {:.2} ns, exceeding 1,000 ns (1 µs) limit",
        nanos_per_keystroke
    );
}

#[test]
fn test_zero_memory_leak_long_stream() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let words = [
        "tieng",
        "viet",
        "nam",
        "doc",
        "lap",
        "tu",
        "do",
        "hanh",
        "phuc",
        "cong",
        "hoa",
        "xa",
        "hoi",
        "chu",
        "nghia",
        "vietkeyra",
        "hyprland",
        "wayland",
        "arch",
        "linux",
    ];
    let tone_keys = ['s', 'f', 'r', 'x', 'j', ' '];

    // Warm up
    for w in words.iter().cycle().take(100) {
        for ch in w.chars() {
            engine.process_char(ch);
        }
        engine.process_char(' ');
    }

    let initial_rss = get_vm_rss_kb();

    // Execute 500,000 realistic keystrokes
    let mut state: u32 = 999;
    for i in 0..500_000 {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        let word = words[(state as usize) % words.len()];
        let ch_idx = (state as usize >> 4) % word.len();
        let ch = word.chars().nth(ch_idx).unwrap_or('a');
        engine.process_char(ch);

        if i % 7 == 0 {
            let tone = tone_keys[(state as usize >> 8) % tone_keys.len()];
            engine.process_char(tone);
        }
        if i % 10 == 0 {
            engine.process_char(' ');
        }
    }

    let final_rss = get_vm_rss_kb();
    println!(
        "Memory Stability: Initial VmRSS = {} kB, Final VmRSS = {} kB, Delta = {} kB",
        initial_rss,
        final_rss,
        final_rss as isize - initial_rss as isize
    );

    // Any memory growth across 500,000 keystrokes should be virtually 0 (buffer capacity is bounded by word length)
    // We allow up to 2MB (2048 kB) for system allocator variations.
    let growth = final_rss.saturating_sub(initial_rss);
    assert!(
        growth < 2048,
        "Memory leak detected! VmRSS grew by {} kB across 500k keystrokes",
        growth
    );
}
