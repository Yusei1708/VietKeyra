use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use vietkeyra_daemon::ipc::{get_socket_path, run_ipc_server};
use vietkeyra_daemon::state::SharedDaemonState;
use vietkeyra_daemon::wayland::run_wayland_thread;

static SHUTDOWN_FLAG: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_signal(sig: libc::c_int) {
    if sig == libc::SIGINT || sig == libc::SIGTERM {
        SHUTDOWN_FLAG.store(true, Ordering::SeqCst);
    }
}

fn install_signal_handlers() {
    unsafe {
        libc::signal(libc::SIGINT, handle_signal as *const () as usize);
        libc::signal(libc::SIGTERM, handle_signal as *const () as usize);
        libc::signal(libc::SIGPIPE, libc::SIG_IGN);
    }
}

fn main() {
    println!("Khởi động VietKeyra Daemon v0.1.0...");
    install_signal_handlers();

    let shutdown = Arc::new(AtomicBool::new(false));
    let shared_state = Arc::new(SharedDaemonState::new());
    let socket_path = get_socket_path();

    let (tx_to_wayland, rx_from_ipc) = channel();
    let (tx_to_ipc, rx_from_wayland) = channel();

    // Spawn Thread 2: Unix Domain Socket IPC Server Thread
    let ipc_shutdown = shutdown.clone();
    let ipc_state = shared_state.clone();
    let ipc_socket = socket_path.clone();
    let ipc_handle = std::thread::spawn(move || {
        if let Err(e) = run_ipc_server(
            ipc_socket,
            ipc_state,
            ipc_shutdown,
            rx_from_wayland,
            tx_to_wayland,
        ) {
            eprintln!("Lỗi IPC Server: {e}");
        }
    });

    // Spawn Thread 1: Wayland IME & Virtual Keyboard Event Loop Thread
    let wayland_shutdown = shutdown.clone();
    let wayland_state = shared_state.clone();
    let wayland_handle = std::thread::spawn(move || {
        run_wayland_thread(wayland_state, wayland_shutdown, rx_from_ipc, tx_to_ipc);
    });

    // Main supervisor loop: monitors global shutdown signal
    while !SHUTDOWN_FLAG.load(Ordering::SeqCst) && !shutdown.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("VietKeyra: Đang dừng daemon và thu dọn tài nguyên...");
    shutdown.store(true, Ordering::SeqCst);

    // Join threads cleanly
    let _ = ipc_handle.join();
    let _ = wayland_handle.join();

    // Ensure socket cleanup
    let _ = std::fs::remove_file(socket_path);
    println!("VietKeyra Daemon đã dừng an toàn.");
}
