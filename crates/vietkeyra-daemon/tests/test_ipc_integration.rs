use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Duration;
use vietkeyra_daemon::ipc::run_ipc_server;
use vietkeyra_daemon::state::SharedDaemonState;

#[test]
fn test_socket_ipc_full_lifecycle() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let socket_path = tmp_dir.path().join("test_vietkeyra.sock");

    let state = Arc::new(SharedDaemonState::new());
    let shutdown = Arc::new(AtomicBool::new(false));
    let (tx_to_wayland, rx_from_ipc) = channel();
    let (tx_to_ipc, rx_from_wayland) = channel();

    let s_path = socket_path.clone();
    let s_state = state.clone();
    let s_shutdown = shutdown.clone();

    let server_handle = std::thread::spawn(move || {
        run_ipc_server(s_path, s_state, s_shutdown, rx_from_wayland, tx_to_wayland).unwrap();
    });

    // Wait for socket to bind
    for _ in 0..50 {
        if socket_path.exists() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(socket_path.exists());

    // 1. Test get_status
    {
        let mut stream = UnixStream::connect(&socket_path).unwrap();
        stream.write_all(b"{\"cmd\": \"get_status\"}\n").unwrap();
        stream.flush().unwrap();

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(line.contains("\"status\":\"ok\""));
        assert!(line.contains("\"mode\":\"VI\""));
        assert!(line.contains("\"engine\":\"telex\""));
    }

    // 2. Test toggle
    {
        let mut stream = UnixStream::connect(&socket_path).unwrap();
        stream.write_all(b"{\"cmd\": \"toggle\"}\n").unwrap();
        stream.flush().unwrap();

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(line.contains("\"status\":\"ok\""));
        assert!(line.contains("\"mode\":\"EN\""));
    }

    // Verify Wayland received SetMode message
    let msg = rx_from_ipc.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(matches!(msg, vietkeyra_daemon::state::IpcToWayland::SetMode(m) if m.is_english()));

    // 3. Test set_mode to VI
    {
        let mut stream = UnixStream::connect(&socket_path).unwrap();
        stream
            .write_all(b"{\"cmd\": \"set_mode\", \"mode\": \"VI\"}\n")
            .unwrap();
        stream.flush().unwrap();

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(line.contains("\"status\":\"ok\""));
        assert!(line.contains("\"mode\":\"VI\""));
    }

    // 4. Test invalid mode error response
    {
        let mut stream = UnixStream::connect(&socket_path).unwrap();
        stream
            .write_all(b"{\"cmd\": \"set_mode\", \"mode\": \"INVALID\"}\n")
            .unwrap();
        stream.flush().unwrap();

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(line.contains("\"status\":\"error\""));
    }

    // 5. Test subscribe streaming
    {
        let mut sub_stream = UnixStream::connect(&socket_path).unwrap();
        sub_stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        sub_stream.write_all(b"{\"cmd\": \"subscribe\"}\n").unwrap();
        sub_stream.flush().unwrap();

        // Allow accept loop to process subscribe connection and register to subscriber pool
        std::thread::sleep(Duration::from_millis(50));

        // Simulate Wayland thread changing mode
        tx_to_ipc
            .send(vietkeyra_daemon::state::WaylandToIpc::ModeChanged(
                vietkeyra_engine::InputMode::English,
            ))
            .unwrap();

        let mut reader = BufReader::new(sub_stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(line.contains("\"event\":\"mode_changed\""));
        assert!(line.contains("\"mode\":\"EN\""));
    }

    // 6. Test clean shutdown and socket unlink
    shutdown.store(true, Ordering::SeqCst);
    server_handle.join().unwrap();
    assert!(!socket_path.exists());
}
