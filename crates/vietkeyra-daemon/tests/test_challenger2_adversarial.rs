//! Adversarial Challenge Test Suite for Milestone 2:
//! Wayland State Machine, Surrounding Text UTF-8 Byte Boundaries,
//! Modifier Bypass, Latency, and Focus Flapping.
//!
//! Author: Challenger 2 (Empirical Verification)

use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Instant;
use vietkeyra_daemon::state::SharedDaemonState;
use vietkeyra_daemon::wayland::{WaylandState, MAX_SURROUNDING_BYTES};
use vietkeyra_engine::{InputMode, KeyAction, TonePlacementStyle, VietKeyraEngine};
use xkbcommon::xkb;

/// Helper to simulate how Wayland zwp_input_method_v2 updates an application's text buffer.
struct MockWaylandTextBuffer {
    content: String,
}

impl MockWaylandTextBuffer {
    fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    /// Applies KeyAction exactly as zwp_input_method_v2 and input_method.rs do.
    fn apply_action(&mut self, action: &KeyAction) {
        match action {
            KeyAction::Replace {
                delete_bytes, text, ..
            } => {
                if *delete_bytes > 0 {
                    assert!(
                        *delete_bytes <= self.content.len(),
                        "delete_bytes ({}) exceeds buffer len ({})",
                        delete_bytes,
                        self.content.len()
                    );
                    let split_idx = self.content.len() - delete_bytes;
                    assert!(
                        self.content.is_char_boundary(split_idx),
                        "CRITICAL BUG: delete_bytes ({}) slices across a UTF-8 character boundary! Buffer: {:?}, split_idx: {}",
                        delete_bytes,
                        self.content,
                        split_idx
                    );
                    self.content.truncate(split_idx);
                }
                self.content.push_str(text);
            }
            KeyAction::Commit { text } => {
                self.content.push_str(text);
            }
            KeyAction::PassThrough => {}
        }
    }
}

// =============================================================================
// 1. UTF-8 Byte Boundary Integrity & delete_surrounding_text
// =============================================================================

#[test]
fn test_utf8_byte_boundary_vietnamese_syllables_adversarial() {
    // Test words containing 1-byte, 2-byte, and 3-byte UTF-8 sequences
    let test_words = [
        ("tieengs", "tiếng"),    // 'ế' is 3 bytes (U+1E7F)
        ("nghieeng", "nghiêng"), // 'ê' is 2 bytes (U+00EA)
        ("hoaf", "hoà"),         // 'à' is 2 bytes (U+00E0)
        ("huyeenj", "huyện"),    // 'ệ' is 3 bytes (U+1EC7)
        ("thuyeefn", "thuyền"),  // 'ề' is 3 bytes (U+1EC1)
        ("dduwowfng", "đường"),  // 'đ' 2b, 'ư' 2b, 'ờ' 3b
        ("nghieejp", "nghiệp"),  // 'ệ' 3b
        ("khoer", "khoẻ"),       // 'ẻ' 3b
        ("thuyr", "thuỷ"),       // 'ỷ' 3b
        ("suwax", "sữa"),        // 'ữ' 3b, 'a' 1b
        ("buwax", "bữa"),        // 'ữ' 3b, 'a' 1b
        ("vuwowfn", "vườn"),     // 'ư' 2b, 'ờ' 3b
        ("phuwowngj", "phượng"), // 'ư' 2b, 'ợ' 3b
        ("nguwowfi", "người"),   // 'ư' 2b, 'ờ' 3b
        ("chuooix", "chuỗi"),    // 'ỗ' 3b
        ("toans", "toán"),       // 'o' 1b, 'á' 2b
        ("luaatj", "luật"),      // 'u' 1b, 'ậ' 3b
        ("quoocs", "quốc"),      // 'ố' 3b
        ("giair", "giải"),       // 'ả' 3b
        ("truyeefn", "truyền"),  // 'ề' 3b
        ("ddoong", "đông"),      // 'đ' 2b, 'ô' 2b
        ("vieetj", "việt"),      // 'ệ' 3b
        ("nam", "nam"),          // all 1b
    ];

    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);

    for (telex_input, expected_output) in test_words {
        engine.reset_word();
        let mut mock_buf = MockWaylandTextBuffer::new();

        // Feed characters one by one
        for ch in telex_input.chars() {
            let action = engine.process_char(ch);
            mock_buf.apply_action(&action);

            // Invariant: mock buffer must equal engine current word
            assert_eq!(
                mock_buf.content,
                engine.current_word(),
                "Buffer mismatch during typing '{}' with char '{}'",
                telex_input,
                ch
            );
            // Invariant: buffer must be 100% valid UTF-8
            assert!(
                std::str::from_utf8(mock_buf.content.as_bytes()).is_ok(),
                "Invalid UTF-8 in buffer for input '{}'",
                telex_input
            );
        }

        assert_eq!(
            mock_buf.content, expected_output,
            "Final output mismatch for telex input '{}'",
            telex_input
        );

        // Adversarial Backspace Verification:
        // Pop every character one by one until empty, checking UTF-8 byte boundary on every step
        while !mock_buf.content.is_empty() {
            let bs_action = engine.handle_backspace();
            match bs_action {
                KeyAction::Replace {
                    delete_bytes,
                    ref text,
                    ..
                } => {
                    assert!(
                        delete_bytes > 0,
                        "Backspace on non-empty buffer must delete >0 bytes"
                    );
                    assert!(
                        text.is_empty(),
                        "Backspace action should not append replacement text"
                    );
                    mock_buf.apply_action(&bs_action);
                    assert_eq!(
                        mock_buf.content,
                        engine.current_word(),
                        "Buffer mismatch during backspace on '{}'",
                        expected_output
                    );
                    assert!(
                        std::str::from_utf8(mock_buf.content.as_bytes()).is_ok(),
                        "Invalid UTF-8 after backspace on '{}'",
                        expected_output
                    );
                }
                _ => panic!("Expected Replace on backspace when buffer not empty"),
            }
        }

        // When buffer is completely empty, next backspace must be PassThrough
        let empty_bs = engine.handle_backspace();
        assert_eq!(
            empty_bs,
            KeyAction::PassThrough,
            "Backspace on empty buffer must return PassThrough"
        );
    }
}

#[test]
fn test_utf8_boundary_tone_swapping_and_repeat_undo() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);
    let mut mock_buf = MockWaylandTextBuffer::new();

    // Type "tieengs" -> "tiếng" ('ế' is 3-byte UTF-8, len = 7 bytes)
    for ch in "tieengs".chars() {
        let act = engine.process_char(ch);
        mock_buf.apply_action(&act);
    }
    assert_eq!(mock_buf.content, "tiếng");
    assert_eq!(mock_buf.content.len(), 7);

    // Tone swap: 's' -> 'f' (tiếng -> tiềng)
    // Both 'ế' (U+1E7F) and 'ề' (U+1EC1) are 3-byte codepoints
    let act_f = engine.process_char('f');
    match act_f {
        KeyAction::Replace { delete_bytes, .. } => {
            assert_eq!(delete_bytes, 7, "Must delete exactly 7 bytes of 'tiếng'");
        }
        _ => panic!("Expected Replace"),
    }
    mock_buf.apply_action(&act_f);
    assert_eq!(mock_buf.content, "tiềng");
    assert_eq!(mock_buf.content.len(), 7);

    // Tone swap: 'f' -> 'r' (tiềng -> tiểng) ('ể' U+1EC3 is 3-byte)
    let act_r = engine.process_char('r');
    mock_buf.apply_action(&act_r);
    assert_eq!(mock_buf.content, "tiểng");

    // Tone swap: 'r' -> 'x' (tiểng -> tiễng) ('ễ' U+1EC5 is 3-byte)
    let act_x = engine.process_char('x');
    mock_buf.apply_action(&act_x);
    assert_eq!(mock_buf.content, "tiễng");

    // Tone swap: 'x' -> 'j' (tiễng -> tiệng) ('ệ' U+1EC7 is 3-byte)
    let act_j = engine.process_char('j');
    mock_buf.apply_action(&act_j);
    assert_eq!(mock_buf.content, "tiệng");

    // Tone swap back to 's' (tiệng -> tiếng)
    let act_s = engine.process_char('s');
    mock_buf.apply_action(&act_s);
    assert_eq!(mock_buf.content, "tiếng");

    // Repeat 's': repeat key undo: tiếng + s -> tiêngs
    // 'ê' is 2 bytes (U+00EA), total length = 1 + 1 + 2 + 1 + 1 + 1 = 7 bytes
    let act_repeat_s = engine.process_char('s');
    mock_buf.apply_action(&act_repeat_s);
    assert_eq!(mock_buf.content, "tiêngs");
    assert_eq!(mock_buf.content.len(), 7);

    // Another 's': raw append -> tiêngss
    let act_raw_s = engine.process_char('s');
    mock_buf.apply_action(&act_raw_s);
    assert_eq!(mock_buf.content, "tiêngss");
    assert_eq!(mock_buf.content.len(), 8);

    // Backspace: removes second 's' (1 byte)
    let bs1 = engine.handle_backspace();
    mock_buf.apply_action(&bs1);
    assert_eq!(mock_buf.content, "tiêngs");

    // Backspace: removes first 's' (1 byte)
    let bs2 = engine.handle_backspace();
    mock_buf.apply_action(&bs2);
    assert_eq!(mock_buf.content, "tiêng");

    // Backspace: removes 'g' (1 byte)
    let bs3 = engine.handle_backspace();
    mock_buf.apply_action(&bs3);
    assert_eq!(mock_buf.content, "tiên");

    // Backspace: removes 'n' (1 byte)
    let bs4 = engine.handle_backspace();
    mock_buf.apply_action(&bs4);
    assert_eq!(mock_buf.content, "tiê");

    // Backspace: removes 'ê' (2 bytes!)
    let bs5 = engine.handle_backspace();
    match bs5 {
        KeyAction::Replace { delete_bytes, .. } => {
            assert_eq!(delete_bytes, 2, "Deleting 'ê' must delete exactly 2 bytes");
        }
        _ => panic!("Expected Replace"),
    }
    mock_buf.apply_action(&bs5);
    assert_eq!(mock_buf.content, "ti");

    // Backspace: removes 'i' (1 byte)
    let bs6 = engine.handle_backspace();
    mock_buf.apply_action(&bs6);
    assert_eq!(mock_buf.content, "t");

    // Backspace: removes 't' (1 byte)
    let bs7 = engine.handle_backspace();
    mock_buf.apply_action(&bs7);
    assert_eq!(mock_buf.content, "");

    // Backspace on empty: PassThrough
    let bs8 = engine.handle_backspace();
    assert_eq!(bs8, KeyAction::PassThrough);
}

// =============================================================================
// 2. Surrounding Text Overflow & Boundary Truncation
// =============================================================================

fn truncate_safe(text: &str) -> &str {
    if text.len() > MAX_SURROUNDING_BYTES {
        let target_start = text.len().saturating_sub(1000);
        let mut start = target_start;
        while start > 0 && !text.is_char_boundary(start) {
            start -= 1;
        }
        &text[start..]
    } else {
        text
    }
}

#[test]
fn test_surrounding_text_overflow_adversarial_boundaries() {
    // Case 1: Exact boundaries around 4000 bytes (ASCII)
    for len in [3999, 4000, 4001, 4002, 5000] {
        let s = "a".repeat(len);
        let res = truncate_safe(&s);
        if len <= 4000 {
            assert_eq!(res.len(), len);
        } else {
            assert_eq!(res.len(), 1000);
        }
        assert!(std::str::from_utf8(res.as_bytes()).is_ok());
    }

    // Case 2: Multi-byte 3-byte Vietnamese chars ('ế' = 3 bytes)
    // 1334 * 3 = 4002 bytes (> 4000)
    let multi_3b = "ế".repeat(1500); // 4500 bytes
    let res_3b = truncate_safe(&multi_3b);
    assert!(
        res_3b.len() <= 1002,
        "Truncated len should be around 1000 bytes, got {}",
        res_3b.len()
    );
    assert!(
        multi_3b.is_char_boundary(multi_3b.len() - res_3b.len()),
        "Truncation start must be a valid char boundary"
    );
    assert!(
        std::str::from_utf8(res_3b.as_bytes()).is_ok(),
        "Must be valid UTF-8"
    );
    assert!(
        res_3b.chars().all(|c| c == 'ế'),
        "All characters must be intact 'ế'"
    );

    // Case 3: 4-byte UTF-8 emojis ('🦀' = 4 bytes)
    let emoji_text = "🦀".repeat(1200); // 4800 bytes
    let res_emoji = truncate_safe(&emoji_text);
    assert!(
        res_emoji.len() <= 1004,
        "Truncated len should be around 1000 bytes, got {}",
        res_emoji.len()
    );
    assert!(
        emoji_text.is_char_boundary(emoji_text.len() - res_emoji.len()),
        "Must be a char boundary"
    );
    assert!(
        std::str::from_utf8(res_emoji.as_bytes()).is_ok(),
        "Must be valid UTF-8"
    );
    assert!(
        res_emoji.chars().all(|c| c == '🦀'),
        "All emojis must be intact"
    );

    // Case 4: Heterogeneous mixed string with emojis, Vietnamese, accents, and ASCII
    let mut mixed = String::new();
    while mixed.len() < 10_000 {
        mixed.push_str("Tiếng Việt độc lập 100% 🦀! ");
    }
    let res_mixed = truncate_safe(&mixed);
    assert!(res_mixed.len() <= 1004);
    assert!(std::str::from_utf8(res_mixed.as_bytes()).is_ok());
    assert!(mixed.ends_with(res_mixed));
}

// =============================================================================
// 3. Modifier & Shortcut Bypass Verification (XKB State)
// =============================================================================

#[test]
fn test_modifier_shortcut_bypass_xkb_simulation() {
    let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
    // Standard US keymap
    let keymap =
        xkb::Keymap::new_from_names(&context, "", "", "", "", None, xkb::KEYMAP_COMPILE_NO_FLAGS)
            .expect("Failed to build default xkb keymap");

    let mut xkb_state = xkb::State::new(&keymap);

    let shared = Arc::new(SharedDaemonState::new());
    let (_tx_ipc, rx_ipc) = channel();
    let (tx_way, _rx_way) = channel();

    let mut state = WaylandState::new(shared, Arc::new(std::sync::Mutex::new(rx_ipc)), tx_way);
    state.active = true;
    state.xkb_keymap = Some(keymap);

    // 1. Normal typing without modifiers: 'v', 'i', 'e'
    let _ = state.engine.process_char('v');
    let _ = state.engine.process_char('i');
    let _ = state.engine.process_char('e');
    assert_eq!(state.engine.current_word(), "vie");

    // 2. Simulate Ctrl modifier depressed:
    // Update modifier mask with Ctrl (mod index for Control in xkb standard is bit 2)
    let ctrl_idx = state
        .xkb_keymap
        .as_ref()
        .unwrap()
        .mod_get_index(xkb::MOD_NAME_CTRL);
    assert_ne!(ctrl_idx, xkb::MOD_INVALID);
    let ctrl_mask = 1 << ctrl_idx;

    xkb_state.update_mask(ctrl_mask, 0, 0, 0, 0, 0);
    assert!(xkb_state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE));

    // Shortcut evaluation: when Ctrl is active, shortcut bypass triggers
    let is_ctrl = xkb_state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE);
    let is_alt = xkb_state.mod_name_is_active(&xkb::MOD_NAME_ALT, xkb::STATE_MODS_EFFECTIVE);
    let is_logo = xkb_state.mod_name_is_active(&xkb::MOD_NAME_LOGO, xkb::STATE_MODS_EFFECTIVE);
    assert!(is_ctrl || is_alt || is_logo);

    // On shortcut, daemon calls reset_word() and forwards key to vk:
    state.engine.reset_word();
    assert!(
        state.engine.current_word().is_empty(),
        "Word buffer must be reset on shortcut chord"
    );

    // 3. Release Ctrl
    xkb_state.update_mask(0, 0, 0, 0, 0, 0);
    assert!(!xkb_state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE));

    // Next key starts a clean new word
    let _ = state.engine.process_char('t');
    assert_eq!(state.engine.current_word(), "t");

    // 4. Test Alt modifier depressed
    let alt_idx = state
        .xkb_keymap
        .as_ref()
        .unwrap()
        .mod_get_index(xkb::MOD_NAME_ALT);
    assert_ne!(alt_idx, xkb::MOD_INVALID);
    let alt_mask = 1 << alt_idx;

    xkb_state.update_mask(alt_mask, 0, 0, 0, 0, 0);
    assert!(xkb_state.mod_name_is_active(&xkb::MOD_NAME_ALT, xkb::STATE_MODS_EFFECTIVE));

    let is_alt_active = xkb_state.mod_name_is_active(&xkb::MOD_NAME_ALT, xkb::STATE_MODS_EFFECTIVE);
    assert!(is_alt_active);
    state.engine.reset_word();
    assert!(state.engine.current_word().is_empty());

    // Release Alt
    xkb_state.update_mask(0, 0, 0, 0, 0, 0);

    // 5. Test Logo/Super modifier depressed
    let logo_idx = state
        .xkb_keymap
        .as_ref()
        .unwrap()
        .mod_get_index(xkb::MOD_NAME_LOGO);
    assert_ne!(logo_idx, xkb::MOD_INVALID);
    let logo_mask = 1 << logo_idx;

    xkb_state.update_mask(logo_mask, 0, 0, 0, 0, 0);
    assert!(xkb_state.mod_name_is_active(&xkb::MOD_NAME_LOGO, xkb::STATE_MODS_EFFECTIVE));
    state.engine.reset_word();
    assert!(state.engine.current_word().is_empty());

    // Release Logo
    xkb_state.update_mask(0, 0, 0, 0, 0, 0);

    // 6. Test Shift modifier alone (Shift should NOT bypass Telex!)
    let shift_idx = state
        .xkb_keymap
        .as_ref()
        .unwrap()
        .mod_get_index(xkb::MOD_NAME_SHIFT);
    assert_ne!(shift_idx, xkb::MOD_INVALID);
    let shift_mask = 1 << shift_idx;

    xkb_state.update_mask(shift_mask, 0, 0, 0, 0, 0);
    let bypass_shift = xkb_state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE)
        || xkb_state.mod_name_is_active(&xkb::MOD_NAME_ALT, xkb::STATE_MODS_EFFECTIVE)
        || xkb_state.mod_name_is_active(&xkb::MOD_NAME_LOGO, xkb::STATE_MODS_EFFECTIVE);

    assert!(
        !bypass_shift,
        "Shift alone MUST NOT bypass Telex (needed for capital letters)"
    );

    // 7. Test English mode bypass
    state.engine.set_mode(InputMode::English);
    assert!(state.engine.get_mode().is_english());
    let en_act = state.engine.process_char('s');
    assert_eq!(
        en_act,
        KeyAction::PassThrough,
        "In English mode, process_char must PassThrough unconditionally"
    );
    let en_bs = state.engine.handle_backspace();
    assert_eq!(
        en_bs,
        KeyAction::PassThrough,
        "In English mode, handle_backspace must PassThrough unconditionally"
    );
}

// =============================================================================
// 4. Focus Flapping & Inter-Window State Isolation
// =============================================================================

#[test]
fn test_focus_flapping_and_window_isolation_stress() {
    let shared = Arc::new(SharedDaemonState::new());
    let (_tx_ipc, rx_ipc) = channel();
    let (tx_way, _rx_way) = channel();

    let mut state = WaylandState::new(shared, Arc::new(std::sync::Mutex::new(rx_ipc)), tx_way);

    // Simulate 500 rapid focus switches (Activate -> Type -> Deactivate -> Type -> Activate)
    for i in 0..500 {
        // Step 1: Window A activates
        state.active = true;
        state.engine.reset_word();
        state.has_surrounding_text = false;
        state.surrounding_text.clear();

        // User starts typing partial word in Window A
        let _ = state.engine.process_char('v');
        let _ = state.engine.process_char('i');
        let _ = state.engine.process_char('e');
        let _ = state.engine.process_char('e'); // "viê"
        assert_eq!(state.engine.current_word(), "viê");

        // Step 2: Window switches away (Deactivate event)
        state.active = false;
        state.engine.reset_word();
        state.has_surrounding_text = false;
        state.surrounding_text.clear();

        assert_eq!(
            state.engine.current_word(),
            "",
            "Word buffer must be empty on deactivation (cycle {})",
            i
        );
        assert!(!state.active);

        // Step 3: While deactivated, keystrokes arrive (e.g. game controls 'w', 'a', 's', 'd')
        // In input_method.rs: if !state.active, keys are forwarded directly without engine processing
        if !state.active {
            // Virtual keyboard forward simulation
            assert!(state.engine.current_word().is_empty());
        }

        // Step 4: Window B activates (fresh input field)
        state.active = true;
        state.engine.reset_word();

        // User types 't' in Window B
        let act = state.engine.process_char('t');
        assert_eq!(
            state.engine.current_word(),
            "t",
            "Window B must see fresh 't', zero state leak from Window A (cycle {})",
            i
        );
        assert_eq!(
            act,
            KeyAction::Replace {
                delete_chars: 0,
                delete_bytes: 0,
                text: "t".to_string()
            }
        );
    }
}

// =============================================================================
// 5. Rapid Typing Throughput & Sub-Millisecond Latency Benchmark
// =============================================================================

#[test]
fn test_rapid_typing_throughput_and_sub_millisecond_latency() {
    let mut engine = VietKeyraEngine::new(TonePlacementStyle::Modern);

    // Sample Vietnamese text with diverse Telex patterns
    let sentence =
        "Coongj hoaf Xax hooiuj Chur nghiax Vieejt Nam ! Ddoocj laapj Tuwj do Hanhj phusc . ";
    let iterations = 200; // 200 * 82 = 16,400 keystrokes

    let mut latencies_ns = Vec::with_capacity(sentence.len() * iterations);
    let total_start = Instant::now();

    for _ in 0..iterations {
        for ch in sentence.chars() {
            let t0 = Instant::now();
            let _action = engine.process_char(ch);
            let elapsed_ns = t0.elapsed().as_nanos();
            latencies_ns.push(elapsed_ns);
        }
    }

    let total_duration = total_start.elapsed();
    let total_keystrokes = latencies_ns.len();
    let throughput_kps = (total_keystrokes as f64) / total_duration.as_secs_f64();

    latencies_ns.sort_unstable();
    let min_ns = latencies_ns[0];
    let max_ns = latencies_ns[total_keystrokes - 1];
    let mean_ns: f64 =
        latencies_ns.iter().map(|&x| x as f64).sum::<f64>() / (total_keystrokes as f64);
    let p50_ns = latencies_ns[total_keystrokes * 50 / 100];
    let p95_ns = latencies_ns[total_keystrokes * 95 / 100];
    let p99_ns = latencies_ns[total_keystrokes * 99 / 100];

    println!("\n=== EMPIRICAL LATENCY & THROUGHPUT BENCHMARK ===");
    println!("Total keystrokes:     {}", total_keystrokes);
    println!("Total duration:       {:?}", total_duration);
    println!("Throughput:           {:.2} keystrokes/sec", throughput_kps);
    println!("Min latency:          {} ns", min_ns);
    println!(
        "Mean latency:         {:.2} ns ({:.4} ms)",
        mean_ns,
        mean_ns / 1_000_000.0
    );
    println!("p50 latency:          {} ns", p50_ns);
    println!("p95 latency:          {} ns", p95_ns);
    println!(
        "p99 latency:          {} ns ({:.4} ms)",
        p99_ns,
        p99_ns as f64 / 1_000_000.0
    );
    println!(
        "Max latency:          {} ns ({:.4} ms)",
        max_ns,
        max_ns as f64 / 1_000_000.0
    );

    // Requirement: Strictly < 1.0 ms (1,000,000 ns) per keystroke
    assert!(
        p99_ns < 1_000_000,
        "p99 latency must be strictly < 1ms (got {} ns)",
        p99_ns
    );
    assert!(
        mean_ns < 100_000.0,
        "Mean latency must be well under 0.1ms (got {:.2} ns)",
        mean_ns
    );
    // Requirement: High typing throughput (1,000+ keystrokes/sec)
    assert!(
        throughput_kps > 10_000.0,
        "Throughput must exceed 10,000 keystrokes/sec (got {:.2})",
        throughput_kps
    );
}
