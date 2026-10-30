use std::path::PathBuf;
use vietkeyra_gui::ui::run_gui;

fn main() -> glib::ExitCode {
    let mut socket_path: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--socket-path" {
            if let Some(val) = args.next() {
                socket_path = Some(PathBuf::from(val));
            }
        }
    }

    run_gui(socket_path)
}
