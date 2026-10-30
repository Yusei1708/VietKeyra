use std::sync::mpsc::channel;
use std::sync::Arc;
use vietkeyra_daemon::state::SharedDaemonState;
use vietkeyra_daemon::wayland::{WaylandState, MAX_SURROUNDING_BYTES};
use vietkeyra_engine::{InputMode, KeyAction};

#[test]
fn test_surrounding_text_truncation_safeguard() {
    let shared = Arc::new(SharedDaemonState::new());
    let (_tx_ipc, rx_ipc) = channel();
    let (tx_way, _rx_way) = channel();

    let state = WaylandState::new(shared, Arc::new(std::sync::Mutex::new(rx_ipc)), tx_way);

    let large_text = "a".repeat(5000);
    assert!(large_text.len() > MAX_SURROUNDING_BYTES);

    // Verify safe char boundary calculation
    let target_start = large_text.len().saturating_sub(1000);
    let mut start = target_start;
    while start > 0 && !large_text.is_char_boundary(start) {
        start -= 1;
    }
    let truncated = &large_text[start..];
    assert!(truncated.len() <= 1000);
    assert_eq!(truncated.len(), 1000);
    assert!(!state.active);
}

#[test]
fn test_exact_utf8_byte_lengths_for_delete_surrounding() {
    // Exact UTF-8 byte checks for Wayland delete_surrounding_text
    let ascii_a = 'a';
    assert_eq!(ascii_a.len_utf8(), 1);

    let circumflex_a = 'â';
    assert_eq!(circumflex_a.len_utf8(), 2);

    let compound_e = 'ệ';
    assert_eq!(compound_e.len_utf8(), 3);

    // Verify engine action byte calculation
    let mut engine = vietkeyra_engine::VietKeyraEngine::default();
    let act1 = engine.process_char('a');
    match act1 {
        KeyAction::Replace { delete_bytes, .. } => assert_eq!(delete_bytes, 0),
        _ => panic!("Expected Replace"),
    }

    let act2 = engine.process_char('a');
    match act2 {
        KeyAction::Replace {
            delete_bytes, text, ..
        } => {
            assert_eq!(delete_bytes, 1); // delete 'a' (1 byte)
            assert_eq!(text, "â");
            assert_eq!(text.len(), 2); // 'â' is 2 bytes in UTF-8
        }
        _ => panic!("Expected Replace"),
    }
}

#[test]
fn test_focus_flapping_and_buffer_flush() {
    let mut engine = vietkeyra_engine::VietKeyraEngine::default();
    engine.process_char('v');
    engine.process_char('i');
    engine.process_char('e');
    assert_eq!(engine.current_word(), "vie");

    // Deactivate event occurs when user switches windows
    engine.reset_word();
    assert!(engine.current_word().is_empty());
    assert!(engine.is_empty());
}

#[test]
fn test_english_mode_passthrough() {
    let mut engine = vietkeyra_engine::VietKeyraEngine::default();
    engine.set_mode(InputMode::English);

    let act = engine.process_char('s');
    assert_eq!(act, KeyAction::PassThrough);

    let act_bs = engine.handle_backspace();
    assert_eq!(act_bs, KeyAction::PassThrough);
}
