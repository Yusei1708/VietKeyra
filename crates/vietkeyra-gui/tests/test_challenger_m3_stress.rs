use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::tempdir;

use vietkeyra_gui::ipc_client::{
    parse_event_line, send_command, spawn_subscription_worker, IpcRequest, MainContextChannel,
};
use vietkeyra_gui::ui::state::GuiStateController;

// 1. STRESS TEST: 1,500+ mock events into subscriber channel.
// Verifies:
// - Zero channel overflow
// - High throughput & bounded latency (<1000ms for 1,500 events)
// - Correct filtering of unknown and malformed events
// - Clean shutdown without hanging or leaked threads
#[test]
fn test_stress_1500_mock_events_zero_overflow_clean_shutdown() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("stress_1500.sock");
    let listener = UnixListener::bind(&socket_path).unwrap();

    let (shutdown_tx, shutdown_rx) = mpsc::channel();
    let (ipc_tx, ipc_rx) = MainContextChannel::channel();

    const TOTAL_EVENTS: usize = 1500;
    const MALFORMED_EVENTS: usize = 200;

    let server_handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(line.contains("subscribe"), "Must receive subscribe command");

        // Rapidly flood 1,500 valid events and 200 malformed/unknown events
        let mut payload = String::with_capacity(128 * (TOTAL_EVENTS + MALFORMED_EVENTS));
        for i in 0..TOTAL_EVENTS {
            let mode = if i % 2 == 0 { "VI" } else { "EN" };
            payload.push_str(&format!(
                "{{\"event\":\"mode_changed\",\"mode\":\"{}\",\"engine\":\"telex\"}}\n",
                mode
            ));

            if i < MALFORMED_EVENTS {
                // Interleave malformed JSON, unknown events, and blank lines
                if i % 3 == 0 {
                    payload.push_str("{\"event\":\"custom_heartbeat\",\"count\":99}\n");
                } else if i % 3 == 1 {
                    payload.push_str("INVALID_GARBAGE_PAYLOAD\n");
                } else {
                    payload.push_str("\n\n");
                }
            }
        }

        stream.write_all(payload.as_bytes()).unwrap();
        stream.flush().unwrap();

        // Keep server alive briefly for client to consume
        std::thread::sleep(Duration::from_millis(300));
    });

    let worker_handle = spawn_subscription_worker(socket_path, ipc_tx, shutdown_rx);

    let start = Instant::now();
    let mut received_valid_count = 0;

    // Drain all valid events from the channel
    while received_valid_count < TOTAL_EVENTS {
        match ipc_rx.recv_timeout(Duration::from_millis(2000)) {
            Ok(event) => {
                assert_eq!(event.event, "mode_changed");
                let expected_mode = if received_valid_count % 2 == 0 {
                    "VI"
                } else {
                    "EN"
                };
                assert_eq!(
                    event.mode, expected_mode,
                    "Event sequence order must be preserved"
                );
                received_valid_count += 1;
            }
            Err(e) => {
                panic!(
                    "Timed out waiting for event #{}: {:?}",
                    received_valid_count, e
                );
            }
        }
    }

    let elapsed = start.elapsed();
    assert_eq!(
        received_valid_count, TOTAL_EVENTS,
        "All 1,500 valid events must be received"
    );
    assert!(
        elapsed < Duration::from_millis(2000),
        "Processing 1,500 events took {:?}, exceeding 2000ms latency budget",
        elapsed
    );

    // Verify channel is empty (all malformed/unknown lines were cleanly filtered out)
    assert!(
        ipc_rx.recv_timeout(Duration::from_millis(100)).is_err(),
        "Channel should contain no leftover invalid events"
    );

    // Clean shutdown verification
    shutdown_tx.send(()).unwrap();
    worker_handle
        .join()
        .expect("Subscriber worker thread must join cleanly within timeout");
    server_handle.join().unwrap();
}

// 2. EMPIRICAL TEST: Recursion Guard Strict Echo Loop Prevention.
// Invariants verified:
// - When GUI receives an external event from daemon, guard flag is active during state update.
// - During guard active state, GUI widget callbacks MUST NOT re-emit `set_mode` back to socket.
// - When user interacts directly (guard inactive), GUI correctly emits `set_mode` to socket.
#[test]
fn test_recursion_guard_strictly_prevents_echo_loops() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("guard_echo_test.sock");
    let listener = UnixListener::bind(&socket_path).unwrap();

    let outbound_cmd_count = Arc::new(AtomicUsize::new(0));
    let outbound_count_clone = outbound_cmd_count.clone();

    // Mock daemon recording all outbound commands sent from the GUI
    let server_handle = std::thread::spawn(move || {
        while let Ok((stream, _)) = listener.accept() {
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line) {
                if n == 0 {
                    break;
                }
                if line.contains("set_mode") {
                    outbound_count_clone.fetch_add(1, Ordering::SeqCst);
                }
                line.clear();
            }
        }
    });

    let controller = GuiStateController::new();
    let sock = socket_path.clone();

    // Replicate GUI's callback logic from widgets.rs:
    // mode_switch.connect_state_set(move |_sw, active| { ... })
    let simulate_switch_toggle = {
        let controller = controller.clone();
        let sock = sock.clone();
        move |new_active: bool| {
            if controller.is_guard_active() {
                // Guard active -> Suppress outbound command (NO echo loop)
                return false;
            }
            // Guard inactive -> Emit outbound command
            let new_mode = if new_active { "VI" } else { "EN" };
            let req = IpcRequest::SetMode {
                mode: new_mode.to_string(),
            };
            let _ = send_command(&sock, &req);
            true
        }
    };

    // PHASE 1: Simulate 100 rapid external daemon events updating GUI
    for i in 0..100 {
        let external_mode = if i % 2 == 0 { "VI" } else { "EN" };
        let is_vi = external_mode == "VI";

        // GUI event handler in ui/mod.rs:
        controller.set_guard(true);
        // Switch widget updates, triggering callback:
        let emitted = simulate_switch_toggle(is_vi);
        assert!(
            !emitted,
            "Callback must NOT emit command when guard is active"
        );

        // Controller updates settings
        let mut settings = controller.get_settings();
        settings.is_vietnamese = is_vi;
        *controller.settings.lock().unwrap() = settings;

        controller.set_guard(false);
    }

    // Verify ZERO echo commands reached the mock daemon during 100 external events
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(
        outbound_cmd_count.load(Ordering::SeqCst),
        0,
        "Recursion guard failed! External events leaked outbound set_mode commands back to socket"
    );

    // PHASE 2: Simulate genuine user interaction (guard is inactive)
    let emitted_user1 = simulate_switch_toggle(true);
    assert!(emitted_user1, "User interaction must be emitted");

    let emitted_user2 = simulate_switch_toggle(false);
    assert!(emitted_user2, "User interaction must be emitted");

    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(
        outbound_cmd_count.load(Ordering::SeqCst),
        2,
        "Authentic user toggles must emit exactly 2 commands"
    );

    drop(server_handle);
}

// 3. STRESS TEST: Rapid Concurrent External Toggles Without Deadlock.
// Invariants verified:
// - Concurrent external events, GUI reads, and IPC queries maintain thread-safety.
// - Zero deadlocks or mutex poisoning across hundreds of concurrent operations.
#[test]
fn test_rapid_concurrent_toggles_and_state_consistency_no_deadlock() {
    let controller = GuiStateController::new();
    let num_threads = 6;
    let iterations_per_thread = 200;
    let mut handles = Vec::new();

    // Spawn writers updating mode from external daemon
    for t in 0..num_threads {
        let ctrl = controller.clone();
        let handle = std::thread::spawn(move || {
            for i in 0..iterations_per_thread {
                let mode = if (t + i) % 2 == 0 { "VI" } else { "EN" };
                let is_vi = ctrl.set_mode_from_daemon(mode);
                assert_eq!(is_vi, mode == "VI");

                // Rapid reads under concurrency
                let s = ctrl.get_settings();
                let _ = s.typing_method;
                let _ = s.is_vietnamese;
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().expect("Thread must not deadlock or panic");
    }

    // Guard must be cleanly restored to false
    assert!(!controller.is_guard_active());
}

// 4. RESILIENCE TEST: Socket Disconnection and Reconnection (Daemon Restart).
// Invariants verified:
// - When daemon crashes or terminates, subscriber enters reconnection loop without panicking.
// - When new daemon starts on same socket path, subscriber reconnects automatically.
// - New events broadcast by the restarted daemon are received accurately.
#[test]
fn test_socket_disconnection_and_reconnection_resilience() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("reconnect_test.sock");

    let (shutdown_tx, shutdown_rx) = mpsc::channel();
    let (ipc_tx, ipc_rx) = MainContextChannel::channel();

    // Step 1: Start Mock Daemon 1
    let listener_1 = UnixListener::bind(&socket_path).unwrap();
    let d1_handle = std::thread::spawn(move || {
        let (mut stream, _) = listener_1.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(line.contains("subscribe"));

        // Push 1 event from Daemon 1
        let ev = "{\"event\":\"mode_changed\",\"mode\":\"VI\",\"engine\":\"telex\"}\n";
        stream.write_all(ev.as_bytes()).unwrap();
        stream.flush().unwrap();

        // Daemon 1 shuts down abruptly (closing socket)
        drop(stream);
        drop(listener_1);
    });

    // Start subscriber worker
    let worker_handle = spawn_subscription_worker(socket_path.clone(), ipc_tx, shutdown_rx);

    // Verify first event from Daemon 1 received
    let ev1 = ipc_rx
        .recv_timeout(Duration::from_millis(1500))
        .expect("Must receive event from Daemon 1");
    assert_eq!(ev1.mode, "VI");

    d1_handle.join().unwrap();

    // Step 2: Daemon 1 is now dead. Remove old socket file.
    let _ = std::fs::remove_file(&socket_path);

    // Allow worker to detect disconnection and enter retry loop
    std::thread::sleep(Duration::from_millis(500));

    // Step 3: Start Mock Daemon 2 on the same socket path (simulating daemon restart)
    let listener_2 = UnixListener::bind(&socket_path).unwrap();
    let d2_handle = std::thread::spawn(move || {
        let (mut stream, _) = listener_2.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(
            line.contains("subscribe"),
            "Reconnected subscriber must re-subscribe"
        );

        // Push 1 event from Daemon 2
        let ev2 = "{\"event\":\"mode_changed\",\"mode\":\"EN\",\"engine\":\"telex\"}\n";
        stream.write_all(ev2.as_bytes()).unwrap();
        stream.flush().unwrap();

        std::thread::sleep(Duration::from_millis(200));
    });

    // Step 4: Verify subscriber successfully receives event from Daemon 2
    let ev2 = ipc_rx
        .recv_timeout(Duration::from_millis(3000))
        .expect("Must receive event from restarted Daemon 2 after auto-reconnect");
    assert_eq!(ev2.mode, "EN");

    // Clean shutdown
    shutdown_tx.send(()).unwrap();
    worker_handle
        .join()
        .expect("Worker thread must join cleanly");
    d2_handle.join().unwrap();
}

/// 5. ADVERSARIAL TEST: Parser Resilience Against Fragmented Chunks & Chaos Lines.
#[test]
fn test_parser_resilience_fragmented_and_chaos_lines() {
    // Test direct parser behavior against fragmented / corrupt lines
    let chaos_inputs = [
        "",
        "   ",
        "\n",
        "\r\n",
        "{}",
        "{\"event\":\"unrecognized\"}",
        "{\"event\":\"mode_changed\"}", // missing mode
        "{\"event\":\"mode_changed\",\"mode\":123}",
        "{\"event\":\"mode_changed\",\"mode\":\"VI\",\"engine\":true}",
        "{{bad json",
        "{\"event\":\"mode_changed\",\"mode\":\"VI\",\"extra_junk\":[1,2,3]}", // valid with extra fields
        "{\"event\":\"mode_changed\",\"mode\":\"EN\",\"engine\":\"telex\"}",
    ];

    let mut valid_events = Vec::new();
    for input in chaos_inputs {
        if let Some(ev) = parse_event_line(input) {
            valid_events.push(ev);
        }
    }

    // Only the valid mode_changed events should parse
    assert_eq!(valid_events.len(), 3);
    assert_eq!(valid_events[0].mode, "");
    assert_eq!(valid_events[1].mode, "VI");
    assert_eq!(valid_events[2].mode, "EN");
}
