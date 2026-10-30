use clap::Parser;
use std::path::{Path, PathBuf};
use std::time::Duration;
use vietkeyra_cli::{connect_to_daemon, format_waybar, get_socket_path, run, send_command, Cli};

#[test]
fn test_challenger_cli_socket_path_resolution() {
    // 1. Explicit override path
    let override_path = PathBuf::from("/custom/runtime/dir/my_vietkeyra.sock");
    assert_eq!(get_socket_path(Some(&override_path)), override_path);

    // 2. Default fallback
    let default_path = get_socket_path(None);
    assert!(
        default_path.to_string_lossy().contains("vietkeyra.sock"),
        "Default path must resolve to a vietkeyra.sock file"
    );
}

#[test]
fn test_challenger_waybar_format_matrix() {
    // Test all valid variants of VI
    for vi_variant in &[
        "VI",
        "vi",
        "Vi",
        "vI",
        "Vietnamese",
        "VIETNAMESE",
        " vietnamese ",
    ] {
        let wb = format_waybar(vi_variant);
        assert_eq!(wb.text, "VI", "Failed for variant: {vi_variant}");
        assert_eq!(wb.class, "vi", "Failed for variant: {vi_variant}");
        assert_eq!(wb.alt, "vietnamese", "Failed for variant: {vi_variant}");
        assert!(wb.tooltip.contains("Tiếng Việt (Telex)"));
        assert!(wb.tooltip.contains("Chuột trái: Đổi sang Tiếng Anh"));
    }

    // Test all valid variants of EN and unexpected input
    for en_variant in &["EN", "en", "English", "unknown", "", "   ", "FR"] {
        let wb = format_waybar(en_variant);
        assert_eq!(wb.text, "EN", "Failed for variant: {en_variant}");
        assert_eq!(wb.class, "en", "Failed for variant: {en_variant}");
        assert_eq!(wb.alt, "english", "Failed for variant: {en_variant}");
        assert!(wb.tooltip.contains("Tiếng Anh (Tắt Telex)"));
        assert!(wb.tooltip.contains("Chuột trái: Đổi sang Tiếng Việt"));
    }
}

#[test]
fn test_challenger_cli_offline_daemon_resilience() {
    let dummy_path = Path::new("/tmp/nonexistent_vietkeyra_999999.sock");

    // 1. connect_to_daemon returns friendly Err without panic
    let conn_res = connect_to_daemon(dummy_path, Duration::from_millis(100));
    assert!(conn_res.is_err());
    let err_msg = conn_res.unwrap_err();
    assert!(err_msg.contains("Không thể kết nối đến vietkeyra-daemon"));
    assert!(err_msg.contains("socket không tồn tại"));

    // 2. send_command returns Err without panic
    let send_res = send_command(dummy_path, "{\"cmd\": \"get_status\"}");
    assert!(send_res.is_err());

    // 3. run(cli) returns Err(1) without panic on offline daemon
    let cli = Cli::parse_from([
        "vietkeyra-cli",
        "--socket-path",
        dummy_path.to_str().unwrap(),
        "status",
    ]);
    let run_res = run(cli);
    assert_eq!(run_res, Err(1));

    let cli_toggle = Cli::parse_from([
        "vietkeyra-cli",
        "--socket-path",
        dummy_path.to_str().unwrap(),
        "toggle",
    ]);
    assert_eq!(run(cli_toggle), Err(1));

    let cli_set = Cli::parse_from([
        "vietkeyra-cli",
        "--socket-path",
        dummy_path.to_str().unwrap(),
        "set-mode",
        "VI",
    ]);
    assert_eq!(run(cli_set), Err(1));

    // Invalid mode argument should return Err(1) immediately without contacting socket
    let cli_invalid = Cli::parse_from([
        "vietkeyra-cli",
        "--socket-path",
        dummy_path.to_str().unwrap(),
        "set-mode",
        "FR",
    ]);
    assert_eq!(run(cli_invalid), Err(1));
}
