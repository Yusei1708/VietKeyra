pub mod input_method;
pub mod virtual_keyboard;

pub use input_method::{WaylandState, MAX_SURROUNDING_BYTES};
pub use virtual_keyboard::VirtualKeyboardHandler;

use crate::state::{IpcToWayland, SharedDaemonState, WaylandToIpc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;
use wayland_client::Connection;

/// Runs the Wayland IME event loop on Thread 1.
pub fn run_wayland_thread(
    shared_state: Arc<SharedDaemonState>,
    shutdown: Arc<AtomicBool>,
    rx_from_ipc: Receiver<IpcToWayland>,
    tx_to_ipc: Sender<WaylandToIpc>,
) {
    let rx_from_ipc = Arc::new(std::sync::Mutex::new(rx_from_ipc));

    while !shutdown.load(Ordering::SeqCst) {
        // Attempt connection to Wayland compositor
        let conn = match Connection::connect_to_env() {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "VietKeyra: Không thể kết nối tới Wayland compositor: {e}. Thử lại sau 2s..."
                );
                // Sleep with shutdown check
                for _ in 0..20 {
                    if shutdown.load(Ordering::SeqCst) {
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                continue;
            }
        };

        let mut event_queue = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut wayland_state =
            WaylandState::new(shared_state.clone(), rx_from_ipc.clone(), tx_to_ipc.clone());

        // Fetch registry globals
        display.get_registry(&qh, ());

        if let Err(e) = event_queue.roundtrip(&mut wayland_state) {
            eprintln!("VietKeyra: Lỗi Wayland initial roundtrip: {e}");
            std::thread::sleep(Duration::from_millis(500));
            return;
        }

        // Bind IME and Virtual Keyboard if compositor globals are present
        if let (Some(seat), Some(im_mgr), Some(vk_mgr)) = (
            wayland_state.seat.as_ref(),
            wayland_state.im_mgr.as_ref(),
            wayland_state.vk_mgr.as_ref(),
        ) {
            let im = im_mgr.get_input_method(seat, &qh, ());
            let grab = im.grab_keyboard(&qh, ());
            let vk = vk_mgr.create_virtual_keyboard(seat, &qh, ());

            wayland_state.im = Some(im);
            wayland_state.grab = Some(grab);
            wayland_state.vk = Some(VirtualKeyboardHandler::new(vk));
        } else {
            eprintln!("VietKeyra Cảnh báo: Compositor không hỗ trợ zwp_input_method_v2 hoặc zwp_virtual_keyboard_v1.");
        }

        // Active dispatch loop
        while !shutdown.load(Ordering::SeqCst) {
            wayland_state.drain_ipc_events();

            match event_queue.blocking_dispatch(&mut wayland_state) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("VietKeyra: Lỗi dispatch Wayland: {e}. Đang tái kết nối...");
                    break;
                }
            }
        }

        // If shutdown was requested, exit cleanly
        if shutdown.load(Ordering::SeqCst) {
            break;
        }

        std::thread::sleep(Duration::from_secs(1));
    }
}
