use clap::Parser;
use vietkeyra_cli::{format_waybar, get_socket_path, Cli, Commands};

#[test]
fn test_cli_subcommands_parse() {
    let cli = Cli::parse_from(["vietkeyra-cli", "status"]);
    match cli.command {
        Commands::Status { waybar, json } => {
            assert!(!waybar);
            assert!(!json);
        }
        _ => panic!("Expected status command"),
    }

    let cli_waybar_flag = Cli::parse_from(["vietkeyra-cli", "status", "--waybar"]);
    match cli_waybar_flag.command {
        Commands::Status { waybar, json } => {
            assert!(waybar);
            assert!(!json);
        }
        _ => panic!("Expected status command with waybar"),
    }

    let cli_waybar = Cli::parse_from(["vietkeyra-cli", "waybar"]);
    assert!(matches!(cli_waybar.command, Commands::Waybar));

    let cli_toggle = Cli::parse_from(["vietkeyra-cli", "toggle"]);
    assert!(matches!(cli_toggle.command, Commands::Toggle));

    let cli_set_mode = Cli::parse_from(["vietkeyra-cli", "set-mode", "VI"]);
    match cli_set_mode.command {
        Commands::SetMode { mode } => assert_eq!(mode, "VI"),
        _ => panic!("Expected set-mode command"),
    }

    let cli_monitor = Cli::parse_from(["vietkeyra-cli", "monitor"]);
    assert!(matches!(cli_monitor.command, Commands::Monitor));
}

#[test]
fn test_waybar_output_vi_and_en() {
    let vi = format_waybar("VI");
    assert_eq!(vi.text, "VI");
    assert_eq!(vi.class, "vi");
    assert_eq!(vi.alt, "vietnamese");
    assert!(vi.tooltip.contains("Tiếng Việt"));

    let vi_json = serde_json::to_string(&vi).unwrap();
    assert!(vi_json.contains(r#""class":"vi""#));
    assert!(vi_json.contains(r#""text":"VI""#));

    let en = format_waybar("EN");
    assert_eq!(en.text, "EN");
    assert_eq!(en.class, "en");
    assert_eq!(en.alt, "english");
    assert!(en.tooltip.contains("Tiếng Anh"));

    let en_json = serde_json::to_string(&en).unwrap();
    assert!(en_json.contains(r#""class":"en""#));
    assert!(en_json.contains(r#""text":"EN""#));
}

#[test]
fn test_socket_path_default() {
    let p = get_socket_path(None);
    assert!(p.to_string_lossy().contains("vietkeyra.sock"));
}
