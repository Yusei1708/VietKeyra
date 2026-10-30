use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::sync::mpsc;
use std::time::Duration;
use tempfile::tempdir;
use vietkeyra_gui::ipc_client::{
    send_command, spawn_subscription_worker, IpcRequest, MainContextChannel,
};
use vietkeyra_gui::ui::state::GuiStateController;

#[test]
fn test_subscriber_worker_receives_events_and_shuts_down_cleanly() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("vietkeyra_sub_test.sock");
    let listener = UnixListener::bind(&socket_path).unwrap();

    let (shutdown_tx, shutdown_rx) = mpsc::channel();
    let (ipc_tx, ipc_rx) = MainContextChannel::channel();

    // Mock daemon accepting subscription and sending push events
    let server_handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(line.contains("subscribe"));

        // Push two mode_changed events
        let ev1 = "{\"event\":\"mode_changed\",\"mode\":\"VI\",\"engine\":\"telex\"}\n";
        stream.write_all(ev1.as_bytes()).unwrap();
        stream.flush().unwrap();

        let ev2 = "{\"event\":\"mode_changed\",\"mode\":\"EN\",\"engine\":\"telex\"}\n";
        stream.write_all(ev2.as_bytes()).unwrap();
        stream.flush().unwrap();

        // Push unknown event (should be ignored cleanly)
        let ev_unknown = "{\"event\":\"unknown_future_event\",\"payload\":123}\n";
        stream.write_all(ev_unknown.as_bytes()).unwrap();
        stream.flush().unwrap();

        // Wait a bit before closing
        std::thread::sleep(Duration::from_millis(50));
    });

    let worker_handle = spawn_subscription_worker(socket_path, ipc_tx, shutdown_rx);

    // Assert that the two valid events are received
    let ev1 = ipc_rx
        .recv_timeout(Duration::from_millis(1500))
        .expect("Should receive first event");
    assert_eq!(ev1.event, "mode_changed");
    assert_eq!(ev1.mode, "VI");

    let ev2 = ipc_rx
        .recv_timeout(Duration::from_millis(1500))
        .expect("Should receive second event");
    assert_eq!(ev2.event, "mode_changed");
    assert_eq!(ev2.mode, "EN");

    // No further events should be queued because unknown event was cleanly dropped
    assert!(ipc_rx.recv_timeout(Duration::from_millis(100)).is_err());

    // Clean shutdown verification: signal subscriber thread to exit
    shutdown_tx.send(()).unwrap();
    worker_handle
        .join()
        .expect("Worker thread must join cleanly without panicking or hanging");
    server_handle.join().unwrap();
}

#[test]
fn test_rapid_toggle_and_guard_flag_invariants() {
    let controller = GuiStateController::new();

    for i in 0..10 {
        let mode = if i % 2 == 0 { "VI" } else { "EN" };
        let is_vi = controller.set_mode_from_daemon(mode);
        assert_eq!(is_vi, mode == "VI");
        assert_eq!(controller.get_settings().is_vietnamese, is_vi);
        assert!(!controller.is_guard_active());
    }
}

#[test]
fn test_all_ipc_commands_against_mock_daemon() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("commands_test.sock");
    let listener = UnixListener::bind(&socket_path).unwrap();

    let server_handle = std::thread::spawn(move || {
        while let Ok((mut stream, _)) = listener.accept() {
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            if reader.read_line(&mut line).is_err() || line.trim().is_empty() {
                break;
            }

            if line.contains("toggle") {
                let resp = "{\"status\":\"ok\",\"mode\":\"EN\",\"engine\":\"telex\",\"version\":\"0.1.0\"}\n";
                let _ = stream.write_all(resp.as_bytes());
            } else if line.contains("get_status")
                || line.contains("set_mode")
                || line.contains("reload_config")
            {
                let resp = "{\"status\":\"ok\",\"mode\":\"VI\",\"engine\":\"telex\",\"version\":\"0.1.0\"}\n";
                let _ = stream.write_all(resp.as_bytes());
            }
            let _ = stream.flush();
        }
    });

    let resp_status = send_command(&socket_path, &IpcRequest::GetStatus).unwrap();
    assert_eq!(resp_status.mode(), Some("VI"));

    let resp_toggle = send_command(&socket_path, &IpcRequest::Toggle).unwrap();
    assert_eq!(resp_toggle.mode(), Some("EN"));

    let resp_set_mode = send_command(
        &socket_path,
        &IpcRequest::SetMode {
            mode: "VI".to_string(),
        },
    )
    .unwrap();
    assert_eq!(resp_set_mode.mode(), Some("VI"));

    let resp_reload = send_command(&socket_path, &IpcRequest::ReloadConfig).unwrap();
    assert!(resp_reload.is_ok());

    drop(server_handle);
}
