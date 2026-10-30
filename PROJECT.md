# Project: VietKeyra

## Architecture
VietKeyra is a 100% independent Vietnamese Telex Input Method Engine (IME) designed natively for Linux (Arch Linux + Hyprland + Waybar).
It operates completely rootless in user session space with zero external IME framework dependencies (no Fcitx5, no IBus).

The system consists of five decoupled components:
1. **`vietkeyra-engine` (Core Telex Engine)**:
   - Pure Rust, zero external dependencies, zero heap allocations on keystroke.
   - Comprehensive NFC Unicode table composition for all 134 vowel-modifier-tone variants.
   - Syllable parser with onset, nucleus, and coda handling.
   - Deterministic tone placement (Modern BGD&ĐT `hoà, thuỷ, khoè` default, configurable Traditional `hòa, thủy, khỏe`).
   - Repeat-key undo / reversion state machine (`aa`->`â`, `aaa`->`aa`, `as`->`á`, `ass`->`as`, `dd`->`đ`, `ddd`->`dd`).
   - Case preservation (UPPERCASE, Titlecase, lowercase, mixed case).
2. **`vietkeyra-daemon` (Wayland Native Daemon)**:
   - Connects to Wayland compositor via `zwp_input_method_v2` (hardware keyboard grab, `commit_string`, `delete_surrounding_text` using UTF-8 byte offsets).
   - Forwards non-Telex keys, hotkeys, and English mode to `zwp_virtual_keyboard_v1`.
   - Multi-tiered fallback: Native Surrounding Text -> Preedit -> Virtual Keyboard backspace emulation.
   - Two-thread decoupled concurrency: Thread 1 runs Wayland event loop + Telex engine (<1ms); Thread 2 runs Unix domain socket IPC.
3. **`vietkeyra-cli` (Command Line & Waybar Driver)**:
   - Interacts with daemon via `$XDG_RUNTIME_DIR/vietkeyra.sock`.
   - Generates JSON for Waybar custom module with instant update via `pkill -RTMIN+8 waybar` (<3ms).
4. **`vietkeyra-gui` (GTK4 Vietnamese Control Panel)**:
   - Pure Rust GTK4 application with idiomatic Vietnamese localization.
   - Asynchronous socket event subscription via `glib::MainContext::channel` for real-time bi-directional synchronization.
5. **System Integration & Tooling (`install.sh`, `uninstall.sh`, Git Security)**:
   - Arch Linux environment detection, active `fcitx5` conflict check and graceful deactivation.
   - Timestamped config backups in `~/.config/vietkeyra/backups/<timestamp>/` with `manifest.json`.
   - Marker-based non-destructive injection and 100% clean uninstallation.
   - Zero-leak Git security: token never written to disk or config; ephemeral auth headers; automated pre-commit hook; strict `.gitignore`.

---

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | F1: Pure Rust Zero-Dependency NFC Telex Engine | 134 vowel-modifier-tone variants, O(1) table composition, zero external normalization crates | M1 | ORIGINAL_REQUEST §R1 |
| 2 | F2: Complete Vietnamese Orthography & Transformations | Modifiers `aa->â, aw->ă, ee->ê, oo->ô, ow->ơ, uw->ư, dd->đ, w->ư`, tones `s,f,r,x,j`, case preservation, free tone typing, repeat-key undo `ss->s, aa->a, dd->d` | M1 | ORIGINAL_REQUEST §R1 |
| 3 | F3: Modern & Traditional Tone Placement | Modern standard `hoà, khoè, thuỷ`, Traditional `hòa, khỏe, thủy`, triphthongs `oai, oay, uôi, ươu, iêu, yêu, uyu` | M1 | ORIGINAL_REQUEST §R1 |
| 4 | F4: 105+ Word Unit Test Suite | Comprehensive unit tests covering all vowels, diphthongs, triphthongs, tones, casing, and undo | M1 | ORIGINAL_REQUEST §Acceptance Criteria |
| 5 | F5: Wayland Input Method Daemon | Native Wayland daemon using `zwp_input_method_v2`, keyboard grab, `commit_string`, UTF-8 byte-length `delete_surrounding_text`, preedit fallback, rootless | M2 | ORIGINAL_REQUEST §R2 |
| 6 | F6: Virtual Keyboard Pass-Through | `zwp_virtual_keyboard_v1` injection for non-consumed keys, shortcuts, and EN mode | M2 | ORIGINAL_REQUEST §R2 |
| 7 | F7: Decoupled Concurrency Architecture | Thread 1 Wayland event loop + Telex engine; Thread 2 Unix domain socket IPC server | M2 | Explorer 2 Survey |
| 8 | F8: Unix Domain Socket IPC Server & Protocol | `$XDG_RUNTIME_DIR/vietkeyra.sock`, JSON Lines schema, commands `get_status, toggle, set_mode, reload_config`, pub/sub broadcast | M2 | ORIGINAL_REQUEST §R2 |
| 9 | F9: CLI Client Tool | `vietkeyra-cli` for IPC status, mode toggle, Waybar JSON formatting, and stream monitoring | M2 | ORIGINAL_REQUEST §R2 |
| 10 | F10: Waybar Custom Module Integration | `custom/vietkeyra` module with `signal: 8`, real-time updates via `pkill -RTMIN+8 waybar` (<3ms), click toggle, right-click GUI, Catppuccin CSS styling | M3 | ORIGINAL_REQUEST §R3 |
| 11 | F11: GTK4 Vietnamese Control Panel GUI | Pure Rust GTK4 app, Vietnamese strings, real-time socket subscription, switch/shortcuts/orthography/guide controls | M3 | ORIGINAL_REQUEST §R4 |
| 12 | F12: Automated Installer Script | `install.sh`: Arch Linux check, `fcitx5` conflict handling, timestamped backup to `~/.config/vietkeyra/backups/<timestamp>/` with `manifest.json`, marker-based injection, rootless | M4 | ORIGINAL_REQUEST §R5 |
| 13 | F13: Automated Uninstaller Script | `uninstall.sh`: stop processes, remove binaries, strip marker blocks, restore configs, zero residue | M4 | ORIGINAL_REQUEST §R5 |
| 14 | F14: Git Repo Setup & Strict Token Security | Git init, remote `https://github.com/Yusei1708/VietKeyra.git`, token never written to disk, ephemeral auth push, `.gitignore` covering `.agents/` and secrets, pre-commit security hook | M4 | ORIGINAL_REQUEST §R6 |
| 15 | F15: E2E Test Suite & Adversarial Hardening | Tiers 1-4 opaque-box tests, 100+ words verification, IPC validation, Wayland simulation, `TEST_READY.md`, Tier 5 adversarial hardening | M5 / E2E Track | ORIGINAL_REQUEST §Acceptance Criteria |

---

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | M1: Core Telex Engine & Vietnamese Orthography | `crates/vietkeyra-engine` (NFC lookup, syllable parser, tone placement, undo state machine, 105+ word test suite) | none | DONE |
| 2 | M2: Wayland Input Method Daemon, Virtual Keyboard & Socket IPC | `crates/vietkeyra-daemon`, `crates/vietkeyra-cli` (`zwp_input_method_v2`, `zwp_virtual_keyboard_v1`, Unix socket IPC `$XDG_RUNTIME_DIR/vietkeyra.sock`) | M1 | DONE |
| 3 | M3: Waybar Integration & GTK4 Vietnamese GUI | `crates/vietkeyra-gui`, Waybar module config (`configs/waybar/`), CSS styling, Hyprland keybind config (`configs/hypr/`) | M2 | DONE |
| 4 | M4: Installer, Uninstaller & Git Token Security | `install.sh`, `uninstall.sh`, `.gitignore`, `scripts/check-secrets.sh`, `.git/hooks/pre-commit`, Git repo initialization and remote push to GitHub | M1, M2, M3 | DONE |
| 5 | M5: Final E2E Test Verification & Adversarial Coverage Hardening | 100% E2E test pass (Tiers 1-4), Tier 5 adversarial hardening, Forensic Audit verification | M1, M2, M3, M4, E2E Track | PLANNED |

---

## Interface Contracts

### M1 `vietkeyra-engine` ↔ M2 `vietkeyra-daemon`
```rust
// In crates/vietkeyra-engine/src/lib.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Vietnamese,
    English,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TonePlacementStyle {
    #[default]
    Modern,      // hoà, thuỷ, khoè
    Traditional, // hòa, thủy, khỏe
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAction {
    PassThrough,
    Replace {
        delete_chars: usize,
        delete_bytes: usize, // Required for Wayland delete_surrounding_text
        text: String,
    },
    Commit {
        text: String,
    },
}

pub struct VietKeyraEngine { /* ... */ }

impl VietKeyraEngine {
    pub fn new(style: TonePlacementStyle) -> Self;
    pub fn set_mode(&mut self, mode: InputMode);
    pub fn get_mode(&self) -> InputMode;
    pub fn process_char(&mut self, ch: char) -> KeyAction;
    pub fn handle_backspace(&mut self) -> KeyAction;
    pub fn reset_word(&mut self);
}
```

### M2 `vietkeyra-daemon` ↔ M3 `vietkeyra-gui` & `vietkeyra-cli` (Unix Socket IPC)
- Socket Path: `$XDG_RUNTIME_DIR/vietkeyra.sock`
- Permissions: `0700`
- Protocol: JSON Lines (UTF-8 newline-delimited)
- Request:
  ```json
  {"cmd": "get_status"}
  {"cmd": "toggle"}
  {"cmd": "set_mode", "mode": "VI" | "EN"}
  {"cmd": "subscribe"}
  ```
- Response:
  ```json
  {"status": "ok", "mode": "VI" | "EN", "engine": "telex", "version": "0.1.0"}
  ```
- Push Notification (to subscribed sockets):
  ```json
  {"event": "mode_changed", "mode": "VI" | "EN", "engine": "telex"}
  ```

### M2 `vietkeyra-cli` ↔ Waybar
- Command: `vietkeyra-cli waybar`
- Signal: `pkill -RTMIN+8 waybar` (signal slot 8)
- Output format:
  ```json
  {"text": "VI", "alt": "vietnamese", "tooltip": "VietKeyra: Tiếng Việt (Telex)\n• Chuột trái: Đổi sang Tiếng Anh\n• Chuột phải: Mở Bảng điều khiển", "class": "vi"}
  ```

---

## Code Layout
```
/home/yusei1708/code/unikey/
├── Cargo.toml                  # Cargo Workspace configuration
├── crates/
│   ├── vietkeyra-engine/       # M1: Pure Rust Telex engine (zero external dependencies)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── nfc.rs
│   │       ├── syllable.rs
│   │       ├── tone.rs
│   │       ├── telex.rs
│   │       ├── undo.rs
│   │       └── tests/
│   │           └── test_105_words.rs
│   ├── vietkeyra-daemon/       # M2: Wayland Input Method Daemon & Socket IPC Server
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── wayland/
│   │       │   ├── mod.rs
│   │       │   ├── input_method.rs
│   │       │   └── virtual_keyboard.rs
│   │       └── ipc/
│   │           ├── mod.rs
│   │           ├── server.rs
│   │           └── protocol.rs
│   ├── vietkeyra-cli/          # M2: CLI client & Waybar JSON emitter
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   └── vietkeyra-gui/          # M3: GTK4 Vietnamese Control Panel
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── ui/
│           └── ipc_client.rs
├── configs/
│   ├── waybar/
│   │   ├── vietkeyra.jsonc     # M3: Waybar custom module definition
│   │   └── vietkeyra.css       # M3: Waybar style definition
│   └── hypr/
│       └── vietkeyra.conf      # M3: Hyprland autostart & keybind snippet
├── scripts/
│   ├── check-secrets.sh        # M4: Standalone secret scanner
│   ├── pre-commit.sh           # M4: Pre-commit hook script
│   └── push.sh                 # M4: Safe Git push with ephemeral token
├── tests/                      # E2E Test Suite (E2E Track)
│   ├── e2e_runner.sh
│   ├── tier1_features/
│   ├── tier2_boundaries/
│   ├── tier3_combinations/
│   └── tier4_realworld/
├── install.sh                  # M4: Automated installer with backups
├── uninstall.sh                # M4: Automated uninstaller with restore
├── .gitignore                  # M4: Strict exclusions preventing secret leakage
└── README.md                   # Complete documentation in Vietnamese
```
