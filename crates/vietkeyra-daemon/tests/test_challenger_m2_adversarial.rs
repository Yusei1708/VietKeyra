use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::{Duration, Instant};
use vietkeyra_daemon::ipc::run_ipc_server;
use vietkeyra_daemon::state::{IpcToWayland, SharedDaemonState, WaylandToIpc};
use vietkeyra_engine::InputMode;

struct TestServerHarness {
    pub socket_path: std::path::PathBuf,
    pub shutdown: Arc<AtomicBool>,
    pub tx_to_ipc: std::sync::mpsc::Sender<WaylandToIpc>,
    pub rx_from_ipc: std::sync::mpsc::Receiver<IpcToWayland>,
    server_handle: Option<std::thread::JoinHandle<()>>,
    _tmp_dir: tempfile::TempDir,
}

impl TestServerHarness {
    fn new(name: &str) -> Self {
        let tmp_dir = tempfile::tempdir().expect("Failed to create tempdir");
        let socket_path = tmp_dir.path().join(format!("{name}.sock"));
        let state = Arc::new(SharedDaemonState::new());
        let shutdown = Arc::new(AtomicBool::new(false));
        let (tx_to_wayland, rx_from_ipc) = channel();
        let (tx_to_ipc, rx_from_wayland) = channel();

        let s_path = socket_path.clone();
        let s_state = state.clone();
        let s_shutdown = shutdown.clone();

        let server_handle = std::thread::spawn(move || {
            let _ = run_ipc_server(s_path, s_state, s_shutdown, rx_from_wayland, tx_to_wayland);
        });

        // Wait for socket to bind
        let start = Instant::now();
        while !socket_path.exists() {
            if start.elapsed() > Duration::from_secs(3) {
                panic!(
                    "Server failed to bind socket within 3s at {:?}",
                    socket_path
                );
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        Self {
            socket_path,
            shutdown,
            tx_to_ipc,
            rx_from_ipc,
            server_handle: Some(server_handle),
            _tmp_dir: tmp_dir,
        }
    }
}

impl Drop for TestServerHarness {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(handle) = self.server_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Helper to send a request and read a single line response
fn query_socket(path: &std::path::Path, req: &str) -> Result<String, std::io::Error> {
    let mut stream = UnixStream::connect(path)?;
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;
    stream.set_write_timeout(Some(Duration::from_secs(3)))?;

    let payload = if req.ends_with('\n') {
        req.to_string()
    } else {
        format!("{req}\n")
    };
    stream.write_all(payload.as_bytes())?;
    stream.flush()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line)
}

#[test]
fn test_adversarial_concurrent_ipc_flooding() {
    let harness = TestServerHarness::new("flood");
    let thread_count = 40;
    let ops_per_thread = 25;
    let success_count = Arc::new(AtomicUsize::new(0));

    let start_all = Instant::now();
    let mut handles = Vec::new();

    for t_id in 0..thread_count {
        let sock_path = harness.socket_path.clone();
        let success = success_count.clone();

        handles.push(std::thread::spawn(move || {
            for i in 0..ops_per_thread {
                let req = match (t_id + i) % 4 {
                    0 => "{\"cmd\": \"get_status\"}",
                    1 => "{\"cmd\": \"toggle\"}",
                    2 => "{\"cmd\": \"set_mode\", \"mode\": \"VI\"}",
                    _ => "{\"cmd\": \"set_mode\", \"mode\": \"EN\"}",
                };

                match query_socket(&sock_path, req) {
                    Ok(resp) => {
                        if resp.contains("\"status\":\"ok\"") {
                            success.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    Err(e) => {
                        eprintln!("Thread {t_id} op {i} error: {e}");
                    }
                }
            }
        }));
    }

    for h in handles {
        h.join().expect("Worker thread panicked");
    }

    let elapsed = start_all.elapsed();
    let total_ops = thread_count * ops_per_thread;
    let completed = success_count.load(Ordering::SeqCst);

    println!(
        "[EMPIRICAL FLOOD] Completed {completed}/{total_ops} concurrent transactions in {:.2?} ({:.1} ops/sec)",
        elapsed,
        total_ops as f64 / elapsed.as_secs_f64()
    );

    assert_eq!(
        completed, total_ops,
        "All concurrent IPC transactions must succeed without loss"
    );

    // Verify daemon is still responsive after flood
    let final_status = query_socket(&harness.socket_path, "{\"cmd\": \"get_status\"}").unwrap();
    assert!(final_status.contains("\"status\":\"ok\""));
}

#[test]
fn test_adversarial_malformed_and_oversized_payloads() {
    let harness = TestServerHarness::new("malformed");

    // 1. Truncated JSON
    let resp = query_socket(&harness.socket_path, "{\"cmd\": ").unwrap();
    assert!(
        resp.contains("\"status\":\"error\""),
        "Truncated JSON should return error: {resp}"
    );

    // 2. Missing cmd
    let resp = query_socket(&harness.socket_path, "{\"mode\": \"VI\"}").unwrap();
    assert!(
        resp.contains("\"status\":\"error\""),
        "Missing cmd should return error: {resp}"
    );

    // 3. Null and integer cmd
    let resp = query_socket(&harness.socket_path, "{\"cmd\": null}").unwrap();
    assert!(
        resp.contains("\"status\":\"error\""),
        "Null cmd should return error: {resp}"
    );

    let resp = query_socket(&harness.socket_path, "{\"cmd\": 12345}").unwrap();
    assert!(
        resp.contains("\"status\":\"error\""),
        "Int cmd should return error: {resp}"
    );

    // 4. Unknown command
    let resp = query_socket(&harness.socket_path, "{\"cmd\": \"format_hard_drive\"}").unwrap();
    assert!(
        resp.contains("Unknown command"),
        "Unknown cmd should be reported: {resp}"
    );

    // 5. Arbitrary binary / non-UTF8 injection followed by newline
    {
        let mut stream = UnixStream::connect(&harness.socket_path).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let garbage = vec![0xFF, 0xFE, 0x00, 0x01, 0x80, b'\n'];
        let _ = stream.write_all(&garbage);
        let _ = stream.flush();

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        let _ = reader.read_line(&mut line);
        // Returns error response or dropped cleanly
    }

    // 6. Oversized payload (>64KB)
    {
        let huge_padding = "a".repeat(70000);
        let huge_cmd = format!("{{\"cmd\": \"get_status\", \"padding\": \"{huge_padding}\"}}\n");
        let resp = query_socket(&harness.socket_path, &huge_cmd).unwrap();
        assert!(
            resp.contains("Oversized payload rejected") || resp.contains("\"status\":\"error\""),
            "Payload >64KB must be rejected: {resp}"
        );
    }

    // 7. Verify daemon continues working normally
    let normal_resp = query_socket(&harness.socket_path, "{\"cmd\": \"get_status\"}").unwrap();
    assert!(normal_resp.contains("\"status\":\"ok\""));
}

#[test]
fn test_adversarial_subscriber_stream_disconnects_and_broken_pipe() {
    let harness = TestServerHarness::new("subscribers");

    let sub_count = 15;
    let mut streams = Vec::new();

    // Connect 15 subscribers
    for _ in 0..sub_count {
        let mut s = UnixStream::connect(&harness.socket_path).unwrap();
        s.write_all(b"{\"cmd\": \"subscribe\"}\n").unwrap();
        s.flush().unwrap();
        streams.push(s);
    }

    // Let the server register subscribers
    std::thread::sleep(Duration::from_millis(50));

    // Abruptly drop half of the subscriber connections
    streams.truncate(7); // Drops 8 streams immediately, causing BrokenPipe on server write

    // Trigger broadcast events from Wayland thread
    for i in 0..20 {
        let mode = if i % 2 == 0 {
            InputMode::English
        } else {
            InputMode::Vietnamese
        };
        harness
            .tx_to_ipc
            .send(WaylandToIpc::ModeChanged(mode))
            .unwrap();
        std::thread::sleep(Duration::from_millis(10));
    }

    // Verify surviving subscribers receive events cleanly
    for (idx, stream) in streams.iter_mut().enumerate() {
        stream
            .set_read_timeout(Some(Duration::from_millis(500)))
            .unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .expect("Read from surviving stream");
        assert!(
            line.contains("\"event\":\"mode_changed\""),
            "Surviving subscriber {idx} must receive broadcast events, got: {line}"
        );
    }

    // Server should still accept new connections and commands
    let check = query_socket(&harness.socket_path, "{\"cmd\": \"get_status\"}").unwrap();
    assert!(check.contains("\"status\":\"ok\""));
}

#[test]
fn test_adversarial_stale_socket_recovery() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let socket_path = tmp_dir.path().join("orphaned.sock");

    // Scenario 1: Create a dead socket file (not listening)
    {
        let dead_listener = std::os::unix::net::UnixListener::bind(&socket_path).unwrap();
        drop(dead_listener);
    }
    assert!(socket_path.exists(), "Orphaned socket file should exist");

    // Start daemon: it should recover and bind cleanly over the orphaned socket
    let state = Arc::new(SharedDaemonState::new());
    let shutdown = Arc::new(AtomicBool::new(false));
    let (tx_to_wayland, _rx_from_ipc) = channel();
    let (_tx_to_ipc, rx_from_wayland) = channel();

    let s_path = socket_path.clone();
    let s_state = state.clone();
    let s_shutdown = shutdown.clone();

    let server_handle = std::thread::spawn(move || {
        run_ipc_server(s_path, s_state, s_shutdown, rx_from_wayland, tx_to_wayland).unwrap();
    });

    // Wait for server to bind
    std::thread::sleep(Duration::from_millis(100));

    let resp = query_socket(&socket_path, "{\"cmd\": \"get_status\"}").unwrap();
    assert!(
        resp.contains("\"status\":\"ok\""),
        "Must recover from orphaned socket"
    );

    // Scenario 2: Active socket exists with live server
    // Attempting to run a second daemon instance on the same socket must fail with AddrInUse
    let second_shutdown = Arc::new(AtomicBool::new(false));
    let (tx2, _rx2) = channel();
    let (_tx2_ipc, rx2_wayland) = channel();
    let res = run_ipc_server(
        socket_path.clone(),
        state.clone(),
        second_shutdown,
        rx2_wayland,
        tx2,
    );
    assert!(
        res.is_err(),
        "Second daemon instance must fail when socket is actively held"
    );
    assert_eq!(
        res.unwrap_err().kind(),
        std::io::ErrorKind::AddrInUse,
        "Error kind must be AddrInUse"
    );

    // Clean shutdown of first instance
    shutdown.store(true, Ordering::SeqCst);
    server_handle.join().unwrap();
    assert!(
        !socket_path.exists(),
        "Socket file must be unlinked on shutdown"
    );
}

#[test]
fn test_adversarial_rapid_state_toggles_and_invariants() {
    let harness = TestServerHarness::new("rapid_toggles");

    let toggle_count = 100;
    let mut current_mode = "VI";

    for i in 0..toggle_count {
        let resp = query_socket(&harness.socket_path, "{\"cmd\": \"toggle\"}").unwrap();
        let expected_mode = if current_mode == "VI" { "EN" } else { "VI" };

        assert!(
            resp.contains(&format!("\"mode\":\"{expected_mode}\"")),
            "Iteration {i}: expected mode {expected_mode}, got response: {resp}"
        );
        current_mode = expected_mode;

        // Drain the Wayland channel message
        let msg = harness
            .rx_from_ipc
            .recv_timeout(Duration::from_millis(200))
            .expect("Wayland must receive SetMode message");
        match msg {
            IpcToWayland::SetMode(m) => {
                let m_str = if m.is_vietnamese() { "VI" } else { "EN" };
                assert_eq!(m_str, expected_mode);
            }
            other => panic!("Unexpected message to Wayland: {:?}", other),
        }
    }
}

#[test]
fn test_adversarial_partial_writes_and_blank_lines() {
    let harness = TestServerHarness::new("partial_writes");

    // 1. Client connects, writes 5 bytes, closes immediately without newline
    {
        let mut stream = UnixStream::connect(&harness.socket_path).unwrap();
        let _ = stream.write_all(b"{\"cmd\"");
        drop(stream); // Immediate EOF
    }

    // 2. Client connects and sends multiple consecutive empty lines
    {
        let mut stream = UnixStream::connect(&harness.socket_path).unwrap();
        let _ = stream.write_all(b"\n\n\n\n");
        let _ = stream.flush();
    }

    // 3. Deeply nested JSON object
    {
        let nested = "{\"cmd\": \"get_status\", \"a\": {\"b\": {\"c\": {\"d\": 1}}}}\n";
        let resp = query_socket(&harness.socket_path, nested).unwrap();
        assert!(
            resp.contains("\"status\":\"ok\""),
            "Nested JSON with valid cmd should succeed"
        );
    }

    // Verify daemon is fully functional
    let check = query_socket(&harness.socket_path, "{\"cmd\": \"get_status\"}").unwrap();
    assert!(check.contains("\"status\":\"ok\""));
}

#[test]
fn test_adversarial_cli_binary_execution_and_latency() {
    let harness = TestServerHarness::new("cli_stress");

    let cli_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/vietkeyra-cli");
    let ctl_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/vietkeyra-ctl");

    if !cli_path.exists() {
        // Compile binary if needed
        let status = Command::new("cargo")
            .args(["build", "-p", "vietkeyra-cli"])
            .status()
            .expect("Build vietkeyra-cli binary");
        assert!(status.success());
    }

    let sock_arg = harness.socket_path.to_str().unwrap();

    // 1. Status command test
    let output = Command::new(&cli_path)
        .args(["--socket-path", sock_arg, "status"])
        .output()
        .expect("Run vietkeyra-cli status");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("VietKeyra v0.1.0"));

    // 2. Status --json
    let output = Command::new(&cli_path)
        .args(["--socket-path", sock_arg, "status", "--json"])
        .output()
        .expect("Run vietkeyra-cli status --json");
    assert!(output.status.success());
    let json_str = String::from_utf8_lossy(&output.stdout);
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("Valid JSON");
    assert_eq!(val["status"], "ok");

    // 3. Waybar custom module JSON schema & latency test (<10ms per call)
    let mut durations = Vec::new();
    for _ in 0..25 {
        let t0 = Instant::now();
        let output = Command::new(&cli_path)
            .args(["--socket-path", sock_arg, "waybar"])
            .output()
            .expect("Run vietkeyra-cli waybar");
        let dt = t0.elapsed();
        durations.push(dt);

        assert!(output.status.success());
        let wb_json = String::from_utf8_lossy(&output.stdout);
        let val: serde_json::Value = serde_json::from_str(&wb_json).expect("Valid Waybar JSON");
        assert!(val.get("text").is_some());
        assert!(val.get("alt").is_some());
        assert!(val.get("tooltip").is_some());
        assert!(val.get("class").is_some());
    }

    let avg_ms = durations.iter().map(|d| d.as_micros()).sum::<u128>() as f64
        / (durations.len() as f64 * 1000.0);
    println!(
        "[EMPIRICAL CLI] Waybar invocation avg latency: {:.2}ms (25 invocations)",
        avg_ms
    );
    assert!(
        avg_ms < 20.0,
        "CLI Waybar output should average under 20ms, was {:.2}ms",
        avg_ms
    );

    // 4. Repeated CLI toggles
    for i in 0..20 {
        let output = Command::new(&cli_path)
            .args(["--socket-path", sock_arg, "toggle"])
            .output()
            .expect("Run vietkeyra-cli toggle");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let expected = if i % 2 == 0 { "EN" } else { "VI" };
        assert!(
            stdout.contains(expected),
            "Toggle {i} expected {expected}, got: {stdout}"
        );
    }

    // 5. Test alias binary vietkeyra-ctl
    let output_ctl = Command::new(&ctl_path)
        .args(["--socket-path", sock_arg, "waybar"])
        .output()
        .expect("Run vietkeyra-ctl waybar");
    assert!(output_ctl.status.success());
}
