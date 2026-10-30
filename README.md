# 🌸 VietKeyra — Bộ gõ Tiếng Việt Độc Lập cho Linux (Wayland / Hyprland / Waybar)

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](#)
[![Tests](https://img.shields.io/badge/tests-173%20passed-success)](#)
[![Platform](https://img.shields.io/badge/platform-Arch%20Linux%20%7C%20Hyprland-blue)](#)
[![Language](https://img.shields.io/badge/language-Rust%20100%25-orange)](#)
[![License](https://img.shields.io/badge/license-MIT-informational)](#)

**VietKeyra** là bộ gõ tiếng Việt độc lập 100% bằng Rust được phát triển riêng cho hệ điều hành Linux (Arch Linux + Hyprland + Waybar). VietKeyra giao tiếp trực tiếp với Wayland compositor qua giao thức native input method (`zwp_input_method_v2`) và bàn phím ảo (`zwp_virtual_keyboard_v1`), hoàn toàn không phụ thuộc vào Fcitx5, IBus hay bất kỳ framework gõ phím cồng kềnh nào khác.

---

## 🌟 Điểm nổi bật

1. **Pure Rust Telex Engine (`vietkeyra-engine`)**:
   - Tự xây dựng 100% thuật toán Telex cho chuẩn tiếng Việt Unicode dựng sẵn (NFC).
   - Bảng tra cứu trực tiếp O(1) cho 134 biến thể nguyên âm - biến âm - dấu thanh, không phân bổ bộ nhớ heap trong quá trình xử lý phím (<1μs).
   - Xử lý đầy đủ biến âm (`aa`->`â`, `aw`->`ă`, `ee`->`ê`, `oo`->`ô`, `ow`->`ơ`, `uw`->`ư`, `dd`->`đ`, `w`->`ư`), dấu thanh (`s, f, r, x, j`), gỡ dấu khi lặp phím (`as`->`á`, `ass`->`as`), gõ tắt và phục hồi từ với Backspace.
   - Hỗ trợ cả 2 chuẩn đặt dấu thanh: Chuẩn Mới BGD&ĐT (`hoà, thuỷ, khoè`) và Chuẩn Truyền thống (`hòa, thủy, khỏe`).

2. **Dịch vụ Wayland Native Daemon (`vietkeyra-daemon`)**:
   - Bắt phím khi cửa sổ nhập liệu focus, xử lý Telex và thay thế ký tự bằng `commit_string` và `delete_surrounding_text` (tính toán chính xác độ dài byte UTF-8).
   - Cơ chế fallback thông minh: Native Surrounding Text -> Preedit string -> Virtual Keyboard backspace emulation.
   - Chạy 100% với quyền người dùng thông thường trong session space, không yêu cầu `root` hay `sudo`.

3. **Giao tiếp Unix Socket IPC thời gian thực**:
   - Socket tại `$XDG_RUNTIME_DIR/vietkeyra.sock` với giao thức JSON Lines.
   - Hỗ trợ đa tiến trình đồng thời: CLI query và GUI subscriber không bao giờ bị nghẽn (decoupled concurrency).

4. **Tích hợp thanh trạng thái Waybar mượt mà**:
   - Module `custom/vietkeyra` hiển thị tức thời `VI` hoặc `EN` với phối màu chuẩn Catppuccin.
   - Phản hồi siêu tốc (<3ms) thông qua tín hiệu thời gian thực `SIGRTMIN+8`.
   - Click chuột trái để đổi ngôn ngữ, click chuột phải để mở Bảng điều khiển GUI.

5. **Giao diện đồ họa GTK4 tiếng Việt (`vietkeyra-gui`)**:
   - Ứng dụng đồ họa trực quan, thuần tiếng Việt.
   - Tự động đồng bộ trạng thái 2 chiều với Daemon qua socket event stream.
   - Bảng tra cứu phím tắt và quy tắc chính tả trực quan.

6. **Trình cài đặt & gỡ cài đặt an toàn (`install.sh`, `uninstall.sh`)**:
   - Tự động nhận diện Arch Linux, kiểm tra xung đột với fcitx5/ibus.
   - Sao lưu tự động toàn bộ cấu hình vào `~/.config/vietkeyra/backups/<timestamp>/` kèm `manifest.json`.
   - Chèn cấu hình bằng các khối Sentinel `# >>> VIETKEYRA START >>>` ... `# <<< VIETKEYRA END <<<`.
   - Gỡ bỏ sạch sẽ 100% không để lại dấu vết.

---

## 🚀 Cài đặt nhanh

### Yêu cầu hệ thống:
- Hệ điều hành: Arch Linux (hoặc các bản phân phối dựa trên Arch như EndeavourOS, Manjaro).
- Compositor: Hyprland.
- Thanh trạng thái: Waybar.
- Toolchain: Rust & Cargo (`rustc >= 1.70`).
- Thư viện: `gtk4` (`pacman -S gtk4`).

### Các bước cài đặt:

```bash
git clone https://github.com/Yusei1708/VietKeyra.git
cd VietKeyra

# Chạy thử mô phỏng kiểm tra (không thay đổi cấu hình hệ thống)
./install.sh --dry-run

# Tiến hành cài đặt chính thức
./install.sh
```

Sau khi cài đặt thành công, các file thực thi sẽ được cài vào `~/.local/bin/`:
- `vietkeyra-daemon`: Dịch vụ bộ gõ Wayland chạy nền.
- `vietkeyra-cli`: Công cụ dòng lệnh và nguồn cấp dữ liệu cho Waybar.
- `vietkeyra-gui`: Giao diện điều khiển đồ họa GTK4.
- `vietkeyra-ctl`: Symlink trỏ đến `vietkeyra-cli`.

---

## ⌨️ Phím tắt & Sử dụng

| Thao tác | Phím tắt / Lệnh | Mô tả |
|---|---|---|
| **Chuyển đổi VI / EN** | `Super + Space` | Đổi qua lại tức thì giữa Tiếng Việt và Tiếng Anh |
| **Mở Bảng điều khiển** | `Super + Shift + V` | Mở giao diện VietKeyra GTK4 |
| **Waybar Chuột trái** | Click vào module `VI`/`EN` | Chuyển đổi ngôn ngữ gõ |
| **Waybar Chuột phải** | Click phải vào module | Mở Bảng điều khiển |
| **CLI Kiểm tra trạng thái** | `vietkeyra-cli status` | Xuất trạng thái JSON hiện tại của daemon |
| **CLI Đổi chế độ** | `vietkeyra-cli toggle` | Đổi trạng thái qua IPC |
| **CLI Đặt chế độ** | `vietkeyra-cli set-mode VI` | Bật chế độ Tiếng Việt (`VI` hoặc `EN`) |

---

## 🛠️ Cấu trúc mã nguồn

```
VietKeyra/
├── Cargo.toml                  # Cargo Workspace configuration
├── crates/
│   ├── vietkeyra-engine/       # M1: Pure Rust Telex engine (zero external dependencies)
│   ├── vietkeyra-daemon/       # M2: Wayland Input Method Daemon & Socket IPC Server
│   ├── vietkeyra-cli/          # M2: CLI client & Waybar JSON emitter
│   └── vietkeyra-gui/          # M3: GTK4 Vietnamese Control Panel
├── configs/
│   ├── waybar/                 # Cấu hình module & CSS cho Waybar
│   ├── hypr/                   # Cấu hình phím tắt & autostart cho Hyprland
│   └── vietkeyra.desktop       # Menu ứng dụng Linux
├── scripts/
│   ├── check-secrets.sh        # Trình quét mã độc lập kiểm tra bí mật & token
│   ├── pre-commit.sh           # Git hook kiểm tra an toàn trước mỗi commit
│   └── push.sh                 # Script đẩy mã nguồn an toàn với ephemeral token
├── tests/                      # Bộ kiểm thử End-to-End 173 tests (Tiers 1-4)
├── install.sh                  # Trình cài đặt tự động
├── uninstall.sh                # Trình gỡ cài đặt hoàn tác 100%
└── README.md
```

---

## 🧪 Kiểm thử & Xác minh

Toàn bộ hệ thống được bảo đảm chất lượng thông qua bộ test 173 trường hợp:

```bash
# Kiểm tra định dạng code Rust
cargo fmt --check

# Kiểm tra chất lượng và cảnh báo nghiêm ngặt
cargo clippy --workspace --all-targets -- -D warnings

# Chạy toàn bộ unit tests và stress tests
cargo test --workspace

# Chạy trọn vẹn bộ kiểm thử End-to-End 4-tier
./tests/e2e_runner.sh

# Quét bảo mật an toàn token
./scripts/check-secrets.sh
```

---

## 🧹 Gỡ cài đặt

Nếu bạn muốn gỡ cài đặt và đưa toàn bộ cấu hình hệ thống về trạng thái nguyên bản:

```bash
./uninstall.sh
```

Hoặc khôi phục chính xác từ bản sao lưu ban đầu:

```bash
./uninstall.sh --restore-backup
```

---

## 📄 Bản quyền

Phát hành theo giấy phép MIT License. Tác giả: Yusei (2026).
