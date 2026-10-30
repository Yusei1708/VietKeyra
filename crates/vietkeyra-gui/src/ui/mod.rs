pub mod state;
pub mod widgets;

use gtk4::prelude::*;
use gtk4::Application;
use std::path::PathBuf;
use std::sync::mpsc;

use crate::ipc_client;
use state::GuiStateController;

/// Main GUI entry point initializing the GTK4 application and event channels.
pub fn run_gui(socket_override: Option<PathBuf>) -> glib::ExitCode {
    let app = Application::builder()
        .application_id("com.github.vietkeyra.gui")
        .build();

    let sock_opt = socket_override;

    app.connect_activate(move |app| {
        let socket_path = ipc_client::get_socket_path(sock_opt.as_deref());
        let state = GuiStateController::new();

        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        let widgets = widgets::create_widgets(app, socket_path.clone(), state.clone(), shutdown_tx);

        let mode_switch = widgets.mode_switch.clone();
        let status_banner = widgets.status_banner.clone();
        let state_clone = state.clone();

        let (ipc_tx, ipc_rx) = ipc_client::MainContextChannel::channel();

        // Attach event receiver to glib MainContext event loop
        glib::timeout_add_local(std::time::Duration::from_millis(20), move || {
            while let Ok(event) = ipc_rx.try_recv() {
                if event.event == "mode_changed" {
                    state_clone.set_guard(true);
                    let is_vi = event.mode.eq_ignore_ascii_case("VI");
                    mode_switch.set_active(is_vi);

                    let mut cur = state_clone.get_settings();
                    cur.is_vietnamese = is_vi;
                    *state_clone.settings.lock().unwrap() = cur;

                    state_clone.set_guard(false);
                    status_banner.set_visible(false);
                }
            }
            glib::ControlFlow::Continue
        });

        // Spawn background thread to listen to real-time socket events
        let _sub_handle = ipc_client::spawn_subscription_worker(socket_path, ipc_tx, shutdown_rx);

        widgets.window.present();
    });

    app.run_with_args(&std::env::args().collect::<Vec<_>>())
}
