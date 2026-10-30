use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(
    name = "vietkeyra-cli",
    author,
    version = "0.1.0",
    about = "VietKeyra Command-Line & Waybar Client"
)]
pub struct Cli {
    #[arg(long, help = "Path to VietKeyra Unix domain socket")]
    pub socket_path: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Get current IME status")]
    Status {
        #[arg(long, help = "Emit Waybar JSON format")]
        waybar: bool,

        #[arg(long, help = "Emit raw JSON status")]
        json: bool,
    },

    #[command(about = "Emit Waybar custom module JSON")]
    Waybar,

    #[command(about = "Toggle input mode between Vietnamese and English")]
    Toggle,

    #[command(about = "Set input mode explicitly (VI or EN)")]
    #[command(name = "set-mode")]
    SetMode {
        #[arg(help = "Input mode (VI or EN)")]
        mode: String,
    },

    #[command(about = "Subscribe to live IME events")]
    Monitor,

    #[command(about = "Quit/exit the VietKeyra daemon process")]
    Quit,

    #[command(about = "Launch or ensure VietKeyra daemon is running (used by desktop launcher)")]
    Launch,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub status: String,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub engine: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WaybarOutput {
    pub text: String,
    pub alt: String,
    pub tooltip: String,
    pub class: String,
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

/// Formats the Waybar JSON object strictly according to project specifications.
pub fn format_waybar(mode: &str) -> WaybarOutput {
    let mode_upper = mode.trim().to_uppercase();
    if mode_upper == "VI" || mode_upper == "VIETNAMESE" {
        WaybarOutput {
            text: "VI".to_string(),
            alt: "vietnamese".to_string(),
            tooltip: "VietKeyra: Tiếng Việt (Telex)\n• Chuột trái: Đổi sang Tiếng Anh\n• Chuột phải: Thoát VietKeyra".to_string(),
            class: "vi".to_string(),
        }
    } else {
        WaybarOutput {
            text: "EN".to_string(),
            alt: "english".to_string(),
            tooltip: "VietKeyra: Tiếng Anh (Tắt Telex)\n• Chuột trái: Đổi sang Tiếng Việt\n• Chuột phải: Thoát VietKeyra".to_string(),
            class: "en".to_string(),
        }
    }
}

/// Connects to the daemon Unix domain socket with a strict timeout.
pub fn connect_to_daemon(socket_path: &Path, timeout: Duration) -> Result<UnixStream, String> {
    if !socket_path.exists() {
        return Err(format!(
            "Không thể kết nối đến vietkeyra-daemon (socket không tồn tại: {:?}). Vui lòng kiểm tra daemon đã chạy chưa.",
            socket_path
        ));
    }

    let stream = UnixStream::connect(socket_path).map_err(|e| {
        format!(
            "Không thể kết nối đến vietkeyra-daemon tại {:?}: {}. Vui lòng kiểm tra daemon đã chạy chưa.",
            socket_path, e
        )
    })?;

    stream
        .set_read_timeout(Some(timeout))
        .map_err(|e| format!("Failed to set read timeout: {e}"))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|e| format!("Failed to set write timeout: {e}"))?;

    Ok(stream)
}

/// Sends a single command and returns the response string.
pub fn send_command(socket_path: &Path, cmd_json: &str) -> Result<String, String> {
    let mut stream = connect_to_daemon(socket_path, Duration::from_millis(2000))?;

    let payload = if cmd_json.ends_with('\n') {
        cmd_json.to_string()
    } else {
        format!("{cmd_json}\n")
    };

    stream
        .write_all(payload.as_bytes())
        .map_err(|e| format!("Lỗi gửi lệnh đến daemon: {e}"))?;
    stream
        .flush()
        .map_err(|e| format!("Lỗi flush socket: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .map_err(|e| format!("Lỗi đọc phản hồi từ daemon: {e}"))?;

    if response_line.is_empty() {
        return Err("Daemon đóng kết nối mà không phản hồi".to_string());
    }

    Ok(response_line.trim().to_string())
}

/// Executes the CLI command and returns an exit code (0 for success, non-zero for error).
pub fn run(cli: Cli) -> Result<(), i32> {
    let socket_path = get_socket_path(cli.socket_path.as_deref());

    match cli.command {
        Commands::Status { waybar, json } => {
            let resp = match send_command(&socket_path, "{\"cmd\": \"get_status\"}") {
                Ok(r) => r,
                Err(err) => {
                    eprintln!("{err}");
                    return Err(1);
                }
            };

            if waybar {
                let status: DaemonStatus = serde_json::from_str(&resp).map_err(|e| {
                    eprintln!("Lỗi giải mã trạng thái từ daemon: {e}");
                    1
                })?;
                let waybar_obj = format_waybar(&status.mode);
                let json_line = serde_json::to_string(&waybar_obj).unwrap_or_default();
                println!("{json_line}");
            } else if json {
                println!("{resp}");
            } else {
                let status: DaemonStatus = serde_json::from_str(&resp).map_err(|e| {
                    eprintln!("Lỗi giải mã trạng thái: {e}");
                    1
                })?;
                println!(
                    "VietKeyra v{} [Mode: {}, Engine: {}]",
                    status.version, status.mode, status.engine
                );
            }
        }

        Commands::Waybar => {
            let resp = match send_command(&socket_path, "{\"cmd\": \"get_status\"}") {
                Ok(r) => r,
                Err(_) => {
                    let waybar_obj = WaybarOutput {
                        text: "--".to_string(),
                        alt: "offline".to_string(),
                        tooltip: "VietKeyra: Đang tắt (Nhấp chuột để khởi động)".to_string(),
                        class: "offline".to_string(),
                    };
                    let json_line = serde_json::to_string(&waybar_obj).unwrap_or_default();
                    println!("{json_line}");
                    return Ok(());
                }
            };

            let status: DaemonStatus = serde_json::from_str(&resp).map_err(|e| {
                eprintln!("Lỗi giải mã trạng thái từ daemon: {e}");
                1
            })?;
            let waybar_obj = format_waybar(&status.mode);
            let json_line = serde_json::to_string(&waybar_obj).unwrap_or_default();
            println!("{json_line}");
        }

        Commands::Toggle => {
            let resp = send_command(&socket_path, "{\"cmd\": \"toggle\"}").map_err(|e| {
                eprintln!("{e}");
                1
            })?;

            let _ = std::process::Command::new("pkill")
                .args(["-RTMIN+8", "waybar"])
                .status();

            let status: Result<DaemonStatus, _> = serde_json::from_str(&resp);
            match status {
                Ok(s) => {
                    println!("VietKeyra: Chuyển sang chế độ {}", s.mode);
                }
                Err(_) => {
                    println!("{resp}");
                }
            }
        }

        Commands::Quit => {
            // Try graceful IPC quit first
            let _ = send_command(&socket_path, "{\"cmd\": \"quit\"}");
            std::thread::sleep(Duration::from_millis(150));
            // Ensure daemon process is stopped
            let _ = std::process::Command::new("pkill")
                .args(["-f", "vietkeyrad"])
                .status();
            // Signal Waybar to refresh status immediately
            let _ = std::process::Command::new("pkill")
                .args(["-RTMIN+8", "waybar"])
                .status();
            // Notification
            let _ = std::process::Command::new("notify-send")
                .args(["-a", "VietKeyra", "VietKeyra", "Đã thoát bộ gõ tiếng Việt"])
                .status();
            println!("VietKeyra: Đã thoát daemon thành công.");
        }

        Commands::Launch => {
            let is_running = send_command(&socket_path, "{\"cmd\": \"get_status\"}").is_ok();
            if is_running {
                let _ = send_command(&socket_path, "{\"cmd\": \"set_mode\", \"mode\": \"VI\"}");
                let _ = std::process::Command::new("pkill")
                    .args(["-RTMIN+8", "waybar"])
                    .status();
                let _ = std::process::Command::new("notify-send")
                    .args(["-a", "VietKeyra", "VietKeyra", "Bộ gõ tiếng Việt đang hoạt động [Chế độ: VI]"])
                    .status();
                println!("VietKeyra: Daemon đang hoạt động [Chế độ: VI]");
            } else {
                let _ = std::process::Command::new("sh")
                    .args(["-c", "nohup vietkeyrad >/dev/null 2>&1 &"])
                    .spawn();
                std::thread::sleep(Duration::from_millis(400));
                let _ = std::process::Command::new("pkill")
                    .args(["-RTMIN+8", "waybar"])
                    .status();
                let _ = std::process::Command::new("notify-send")
                    .args(["-a", "VietKeyra", "VietKeyra", "Đã khởi động bộ gõ tiếng Việt (Telex)"])
                    .status();
                println!("VietKeyra: Đã khởi động daemon");
            }
        }

        Commands::SetMode { mode } => {
            let mode_upper = mode.trim().to_uppercase();
            if mode_upper != "VI" && mode_upper != "EN" {
                eprintln!("Lỗi: Chế độ không hợp lệ '{mode}'. Vui lòng chọn 'VI' hoặc 'EN'.");
                return Err(1);
            }

            let cmd = format!("{{\"cmd\": \"set_mode\", \"mode\": \"{mode_upper}\"}}");
            let resp = match send_command(&socket_path, &cmd) {
                Ok(r) => r,
                Err(err) => {
                    eprintln!("{err}");
                    return Err(1);
                }
            };

            let _ = std::process::Command::new("pkill")
                .args(["-RTMIN+8", "waybar"])
                .status();

            let status: Result<DaemonStatus, _> = serde_json::from_str(&resp);
            match status {
                Ok(s) => {
                    println!("VietKeyra: Đã đặt chế độ {}", s.mode);
                }
                Err(_) => {
                    println!("{resp}");
                }
            }
        }

        Commands::Monitor => {
            let mut stream = match connect_to_daemon(&socket_path, Duration::from_secs(3600)) {
                Ok(s) => s,
                Err(err) => {
                    eprintln!("{err}");
                    return Err(1);
                }
            };

            // Remove read timeout for long streaming
            let _ = stream.set_read_timeout(None);

            if let Err(e) = stream.write_all(b"{\"cmd\": \"subscribe\"}\n") {
                eprintln!("Lỗi đăng ký theo dõi sự kiện: {e}");
                return Err(1);
            }
            let _ = stream.flush();

            let reader = BufReader::new(stream);
            for line_res in reader.lines() {
                match line_res {
                    Ok(line) => {
                        if !line.trim().is_empty() {
                            println!("{line}");
                        }
                    }
                    Err(e) => {
                        eprintln!("Mất kết nối với daemon: {e}");
                        return Err(1);
                    }
                }
            }
        }
    }

    Ok(())
}
