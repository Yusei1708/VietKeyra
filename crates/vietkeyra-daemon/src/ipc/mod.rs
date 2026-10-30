pub mod protocol;
pub mod server;

pub use protocol::{IpcEvent, IpcRequest, IpcResponse, MAX_PAYLOAD_BYTES};
pub use server::{get_socket_path, run_ipc_server, trigger_waybar_signal};
