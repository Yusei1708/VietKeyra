use super::protocol::{parse_request, IpcEvent, IpcRequest, IpcResponse};
use crate::state::{IpcToWayland, SharedDaemonState, WaylandToIpc};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;
use vietkeyra_engine::InputMode;

/// Resolves the socket path, prioritizing $XDG_RUNTIME_DIR/vietkeyra.sock.
pub fn get_socket_path() -> PathBuf {
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

/// Triggers Waybar update signal (RTMIN+8).
pub fn trigger_waybar_signal() {
    let _ = Command::new("pkill").args(["-RTMIN+8", "waybar"]).spawn();
}

/// Manages active subscriber streams and broadcasts events.
struct SubscriberPool {
    subscribers: Vec<UnixStream>,
}

impl SubscriberPool {
    fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    fn add(&mut self, stream: UnixStream) {
        let _ = stream.set_nonblocking(true);
        self.subscribers.push(stream);
    }

    fn broadcast(&mut self, event: &IpcEvent) {
        let payload = event.to_json_line();
        let bytes = payload.as_bytes();

        self.subscribers.retain_mut(|client| {
            match client.write_all(bytes) {
                Ok(_) => {
                    let _ = client.flush();
                    true
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => true,
                Err(_) => false, // Disconnected or broken pipe
            }
        });
    }
}

/// Runs the IPC server on Thread 2.
pub fn run_ipc_server(
    socket_path: PathBuf,
    state: Arc<SharedDaemonState>,
    shutdown: Arc<AtomicBool>,
    rx_from_wayland: Receiver<WaylandToIpc>,
    tx_to_wayland: Sender<IpcToWayland>,
) -> std::io::Result<()> {
    // 1. Stale socket check & recovery
    if socket_path.exists() {
        match UnixStream::connect(&socket_path) {
            Ok(_) => {
                eprintln!(
                    "Lỗi: Một phiên bản vietkeyra-daemon khác đang chạy tại {:?}",
                    socket_path
                );
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AddrInUse,
                    "Another daemon instance is already running",
                ));
            }
            Err(_) => {
                // Previous daemon crashed or stale file, remove it cleanly
                let _ = std::fs::remove_file(&socket_path);
            }
        }
    }

    // Ensure parent directory exists
    if let Some(parent) = socket_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // 2. Bind Unix domain socket listener
    let listener = UnixListener::bind(&socket_path)?;

    // 3. Set file permissions strictly to 0700 (owner only)
    let _ = std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o700));

    // 4. Set listener to nonblocking for responsive accept loop
    listener.set_nonblocking(true)?;

    let mut pool = SubscriberPool::new();

    // 5. Main IPC server loop
    while !shutdown.load(Ordering::SeqCst) {
        // A. Drain events from Wayland IME thread
        while let Ok(msg) = rx_from_wayland.try_recv() {
            match msg {
                WaylandToIpc::ModeChanged(mode) => {
                    let mode_str = match mode {
                        InputMode::Vietnamese => "VI",
                        InputMode::English => "EN",
                    };
                    state.set_mode(mode);
                    pool.broadcast(&IpcEvent::mode_changed(mode_str, state.engine()));
                    trigger_waybar_signal();
                }
            }
        }

        // B. Accept new incoming client connections
        match listener.accept() {
            Ok((stream, _)) => {
                handle_incoming_connection(stream, &state, &mut pool, &tx_to_wayland);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Sleep briefly to avoid busy loop
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(e) => {
                eprintln!("Lỗi accept socket: {e}");
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }

    // 6. Clean up socket file on shutdown
    let _ = std::fs::remove_file(&socket_path);
    Ok(())
}

fn handle_incoming_connection(
    mut stream: UnixStream,
    state: &Arc<SharedDaemonState>,
    pool: &mut SubscriberPool,
    tx_to_wayland: &Sender<IpcToWayland>,
) {
    // Read the first line of the request with timeout
    let _ = stream.set_read_timeout(Some(Duration::from_millis(2000)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(2000)));

    let mut reader = BufReader::new(match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    });

    let mut line = String::new();
    if reader.read_line(&mut line).is_err() || line.trim().is_empty() {
        return;
    }

    match parse_request(&line) {
        Ok(IpcRequest::GetStatus) => {
            let resp = IpcResponse::ok(state.get_mode_str(), state.engine(), state.version());
            let _ = stream.write_all(resp.to_json_line().as_bytes());
            let _ = stream.flush();
        }

        Ok(IpcRequest::Toggle) => {
            let new_mode = state.toggle_mode();
            let mode_str = match new_mode {
                InputMode::Vietnamese => "VI",
                InputMode::English => "EN",
            };
            let _ = tx_to_wayland.send(IpcToWayland::SetMode(new_mode));
            pool.broadcast(&IpcEvent::mode_changed(mode_str, state.engine()));
            trigger_waybar_signal();

            let resp = IpcResponse::ok(mode_str, state.engine(), state.version());
            let _ = stream.write_all(resp.to_json_line().as_bytes());
            let _ = stream.flush();
        }

        Ok(IpcRequest::SetMode { mode }) => {
            let mode_upper = mode.trim().to_uppercase();
            let new_mode = match mode_upper.as_str() {
                "VI" | "VIETNAMESE" => Some(InputMode::Vietnamese),
                "EN" | "ENGLISH" => Some(InputMode::English),
                _ => None,
            };

            if let Some(m) = new_mode {
                state.set_mode(m);
                let mode_str = match m {
                    InputMode::Vietnamese => "VI",
                    InputMode::English => "EN",
                };
                let _ = tx_to_wayland.send(IpcToWayland::SetMode(m));
                pool.broadcast(&IpcEvent::mode_changed(mode_str, state.engine()));
                trigger_waybar_signal();

                let resp = IpcResponse::ok(mode_str, state.engine(), state.version());
                let _ = stream.write_all(resp.to_json_line().as_bytes());
                let _ = stream.flush();
            } else {
                let resp = IpcResponse::error("Invalid mode: expected VI or EN");
                let _ = stream.write_all(resp.to_json_line().as_bytes());
                let _ = stream.flush();
            }
        }

        Ok(IpcRequest::SetEngine { engine }) => {
            let resp = IpcResponse::ok(state.get_mode_str(), &engine, state.version());
            let _ = stream.write_all(resp.to_json_line().as_bytes());
            let _ = stream.flush();
        }

        Ok(IpcRequest::ReloadConfig) => {
            let resp = IpcResponse::ok(state.get_mode_str(), state.engine(), state.version());
            let _ = stream.write_all(resp.to_json_line().as_bytes());
            let _ = stream.flush();
        }

        Ok(IpcRequest::Subscribe) => {
            // Remove timeout for subscription streaming
            let _ = stream.set_read_timeout(None);
            pool.add(stream);
        }

        Err(err_resp) => {
            let _ = stream.write_all(err_resp.to_json_line().as_bytes());
            let _ = stream.flush();
        }
    }
}
