use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::time::Duration;

/// Commands supported by the VietKeyra IPC daemon.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum IpcRequest {
    GetStatus,
    Toggle,
    SetMode { mode: String },
    SetEngine { engine: String },
    ReloadConfig,
    Subscribe,
}

impl IpcRequest {
    pub fn to_json_line(&self) -> String {
        let mut s = serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string());
        s.push('\n');
        s
    }
}

/// Structured response received from daemon.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum IpcResponse {
    Ok {
        status: String,
        mode: String,
        engine: String,
        version: String,
    },
    Error {
        status: String,
        message: String,
    },
}

impl IpcResponse {
    pub fn is_ok(&self) -> bool {
        matches!(self, IpcResponse::Ok { .. })
    }

    pub fn mode(&self) -> Option<&str> {
        match self {
            IpcResponse::Ok { mode, .. } => Some(mode.as_str()),
            _ => None,
        }
    }
}

/// Push notification event broadcast by daemon to subscribers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IpcEvent {
    pub event: String,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub engine: String,
}

/// Resolves the socket path, prioritizing $XDG_RUNTIME_DIR/vietkeyra.sock.
pub fn get_socket_path(override_path: Option<&Path>) -> PathBuf {
    if let Some(p) = override_path {
        return p.to_path_buf();
    }

    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        if !runtime_dir.is_empty() {
            return PathBuf::from(runtime_dir).join("vietkeyra.sock");
        }
    }

    let uid = unsafe { libc::getuid() };
    let run_user = format!("/run/user/{}/vietkeyra.sock", uid);
    if Path::new(&format!("/run/user/{}", uid)).exists() {
        return PathBuf::from(run_user);
    }

    PathBuf::from("/tmp/vietkeyra.sock")
}

/// Connects to daemon with timeout and sends a single command, returning the response.
pub fn send_command(socket_path: &Path, req: &IpcRequest) -> Result<IpcResponse, String> {
    if !socket_path.exists() {
        return Err(format!(
            "Không thể kết nối đến vietkeyra-daemon (socket không tồn tại: {:?})",
            socket_path
        ));
    }

    let stream = UnixStream::connect(socket_path).map_err(|e| {
        format!(
            "Không thể kết nối đến vietkeyra-daemon tại {:?}: {}",
            socket_path, e
        )
    })?;

    stream
        .set_read_timeout(Some(Duration::from_millis(1500)))
        .map_err(|e| format!("Lỗi thiết lập read timeout: {e}"))?;
    stream
        .set_write_timeout(Some(Duration::from_millis(1500)))
        .map_err(|e| format!("Lỗi thiết lập write timeout: {e}"))?;

    let mut stream_writer = stream;
    let payload = req.to_json_line();
    stream_writer
        .write_all(payload.as_bytes())
        .map_err(|e| format!("Lỗi gửi dữ liệu đến socket: {e}"))?;
    stream_writer
        .flush()
        .map_err(|e| format!("Lỗi flush socket: {e}"))?;

    let mut reader = BufReader::new(stream_writer);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .map_err(|e| format!("Lỗi đọc phản hồi từ daemon: {e}"))?;

    if response_line.trim().is_empty() {
        return Err("Daemon đóng kết nối mà không phản hồi".to_string());
    }

    serde_json::from_str::<IpcResponse>(response_line.trim()).map_err(|e| {
        format!(
            "Lỗi giải mã phản hồi JSON ('{}'): {}",
            response_line.trim(),
            e
        )
    })
}

/// Parses an event line, ignoring unrecognized or malformed events cleanly.
pub fn parse_event_line(line: &str) -> Option<IpcEvent> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Try parsing as standard IpcEvent
    if let Ok(event) = serde_json::from_str::<IpcEvent>(trimmed) {
        if event.event == "mode_changed" {
            return Some(event);
        }
    }

    // Safely ignore unrecognized events without desynchronization or panicking
    None
}

/// Channel bridge abstraction providing glib::MainContext channel semantics.
pub struct MainContextChannel;

impl MainContextChannel {
    /// Creates an IPC event channel bridge connecting background subscribers to glib::MainContext.
    pub fn channel() -> (
        std::sync::mpsc::Sender<IpcEvent>,
        std::sync::mpsc::Receiver<IpcEvent>,
    ) {
        std::sync::mpsc::channel()
    }
}

/// Spawns a background worker thread that subscribes to daemon push events
/// and pushes updates to the GTK main loop via a channel.
/// Cleanly exits when shutdown_rx receives a signal or hangs up.
pub fn spawn_subscription_worker(
    socket_path: PathBuf,
    sender: std::sync::mpsc::Sender<IpcEvent>,
    shutdown_rx: Receiver<()>,
) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("vietkeyra-gui-ipc-subscriber".to_string())
        .spawn(move || {
            while shutdown_rx.try_recv().is_err() {
                // If socket doesn't exist yet, wait and retry
                if !socket_path.exists() {
                    for _ in 0..5 {
                        if shutdown_rx.try_recv().is_ok() {
                            return;
                        }
                        std::thread::sleep(Duration::from_millis(200));
                    }
                    continue;
                }

                let stream_res = UnixStream::connect(&socket_path);
                let stream = match stream_res {
                    Ok(s) => s,
                    Err(_) => {
                        for _ in 0..5 {
                            if shutdown_rx.try_recv().is_ok() {
                                return;
                            }
                            std::thread::sleep(Duration::from_millis(200));
                        }
                        continue;
                    }
                };

                // Read timeout so we periodically check shutdown_rx
                let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                let _ = stream.set_write_timeout(Some(Duration::from_millis(1000)));

                let mut stream_writer = stream;
                let sub_payload = IpcRequest::Subscribe.to_json_line();
                if stream_writer.write_all(sub_payload.as_bytes()).is_err()
                    || stream_writer.flush().is_err()
                {
                    continue;
                }

                let mut reader = BufReader::new(stream_writer);
                let mut line = String::new();

                loop {
                    if shutdown_rx.try_recv().is_ok() {
                        return;
                    }

                    line.clear();
                    match reader.read_line(&mut line) {
                        Ok(0) => {
                            // Server closed connection
                            break;
                        }
                        Ok(_) => {
                            if let Some(event) = parse_event_line(&line) {
                                if sender.send(event).is_err() {
                                    // Receiver was dropped (app exiting)
                                    return;
                                }
                            }
                        }
                        Err(ref e)
                            if e.kind() == std::io::ErrorKind::WouldBlock
                                || e.kind() == std::io::ErrorKind::TimedOut =>
                        {
                            // Expected read timeout, loop around to check shutdown
                            continue;
                        }
                        Err(_) => {
                            // Connection broken, attempt reconnect
                            break;
                        }
                    }
                }

                // Short backoff before reconnecting
                std::thread::sleep(Duration::from_millis(300));
            }
        })
        .expect("Failed to spawn IPC subscriber thread")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;
    use std::os::unix::net::UnixListener;
    use tempfile::tempdir;

    #[test]
    fn test_ipc_request_serialization() {
        let req1 = IpcRequest::GetStatus;
        assert_eq!(req1.to_json_line(), "{\"cmd\":\"get_status\"}\n");

        let req2 = IpcRequest::Toggle;
        assert_eq!(req2.to_json_line(), "{\"cmd\":\"toggle\"}\n");

        let req3 = IpcRequest::SetMode {
            mode: "EN".to_string(),
        };
        assert_eq!(
            req3.to_json_line(),
            "{\"cmd\":\"set_mode\",\"mode\":\"EN\"}\n"
        );

        let req4 = IpcRequest::Subscribe;
        assert_eq!(req4.to_json_line(), "{\"cmd\":\"subscribe\"}\n");
    }

    #[test]
    fn test_ipc_response_deserialization() {
        let ok_raw =
            "{\"status\":\"ok\",\"mode\":\"VI\",\"engine\":\"telex\",\"version\":\"0.1.0\"}";
        let parsed: IpcResponse = serde_json::from_str(ok_raw).unwrap();
        assert!(parsed.is_ok());
        assert_eq!(parsed.mode(), Some("VI"));

        let err_raw = "{\"status\":\"error\",\"message\":\"Unknown command\"}";
        let parsed_err: IpcResponse = serde_json::from_str(err_raw).unwrap();
        assert!(!parsed_err.is_ok());
        assert_eq!(parsed_err.mode(), None);
    }

    #[test]
    fn test_parse_event_line() {
        // Valid mode_changed event
        let valid_line = "{\"event\":\"mode_changed\",\"mode\":\"EN\",\"engine\":\"telex\"}\n";
        let ev = parse_event_line(valid_line).expect("Should parse valid event");
        assert_eq!(ev.event, "mode_changed");
        assert_eq!(ev.mode, "EN");
        assert_eq!(ev.engine, "telex");

        // Ignored unknown future event (test T2_F11_2)
        let unknown_line = "{\"event\":\"unknown_future_event\",\"payload\":123}\n";
        assert!(parse_event_line(unknown_line).is_none());

        // Malformed line
        assert!(parse_event_line("random invalid data\n").is_none());
        assert!(parse_event_line("").is_none());
    }

    #[test]
    fn test_send_command_against_mock_daemon() {
        let dir = tempdir().unwrap();
        let sock_path = dir.path().join("test_daemon.sock");
        let listener = UnixListener::bind(&sock_path).unwrap();

        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();

            if line.contains("get_status") {
                let resp = "{\"status\":\"ok\",\"mode\":\"VI\",\"engine\":\"telex\",\"version\":\"0.1.0\"}\n";
                stream.write_all(resp.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
        });

        let resp = send_command(&sock_path, &IpcRequest::GetStatus).unwrap();
        assert!(resp.is_ok());
        assert_eq!(resp.mode(), Some("VI"));

        handle.join().unwrap();
    }

    #[test]
    fn test_send_command_offline_daemon_returns_graceful_error() {
        let dir = tempdir().unwrap();
        let sock_path = dir.path().join("non_existent.sock");

        let res = send_command(&sock_path, &IpcRequest::GetStatus);
        assert!(res.is_err());
        let err_msg = res.unwrap_err();
        assert!(err_msg.contains("Không thể kết nối đến vietkeyra-daemon"));
    }
}
