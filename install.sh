#!/usr/bin/env bash
# ==============================================================================
# install.sh
# Automated Installer for VietKeyra: Native Vietnamese IME for Linux
# (Arch Linux + Hyprland + Waybar)
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"

# Defaults & Flags
DRY_RUN=false
NON_INTERACTIVE=false
KILL_CONFLICTS=false

print_usage() {
    cat <<EOF
VietKeyra Installer - Trình cài đặt Bộ gõ Tiếng Việt cho Linux

Sử dụng:
  ./install.sh [TÙY CHỌN]

Tùy chọn:
  --dry-run              Chạy thử mô phỏng mà không ghi đè cấu hình hay cài đặt binary
  -y, --yes              Chế độ tự động (không nhắc hỏi tương tác)
  --kill-conflicts       Tự động đóng các bộ gõ xung đột (fcitx5, ibus)
  -h, --help             Hiển thị hướng dẫn sử dụng này

Ví dụ:
  ./install.sh --dry-run
  ./install.sh -y
EOF
}

# Parse Command-Line Arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -y|--yes)
            NON_INTERACTIVE=true
            shift
            ;;
        --kill-conflicts|--force)
            KILL_CONFLICTS=true
            shift
            ;;
        -h|--help)
            print_usage
            exit 0
            ;;
        *)
            echo "❌ Tham số không hợp lệ: $1" >&2
            print_usage
            exit 1
            ;;
    esac
done

echo "=============================================================================="
echo "          🌸 CÀI ĐẶT BỘ GÕ TIẾNG VIỆT VIETKEYRA CHO HYPRLAND / WAYBAR 🌸"
echo "=============================================================================="

# ------------------------------------------------------------------------------
# 1. Environment Verification: Arch Linux & Dependencies
# ------------------------------------------------------------------------------
echo "🔍 [1/7] Kiểm tra môi trường hệ điều hành..."
IS_ARCH=false
if [ -f /etc/arch-release ]; then
    IS_ARCH=true
elif [ -f /etc/os-release ]; then
    if grep -qiE "^(ID|ID_LIKE)=.*(arch|manjaro|endeavouros)" /etc/os-release; then
        IS_ARCH=true
    fi
fi

if [ "$IS_ARCH" = false ]; then
    echo "❌ Lỗi: VietKeyra được thiết kế tối ưu dành riêng cho Arch Linux (và hệ phái Arch: EndeavourOS, Manjaro)." >&2
    echo "   Hệ điều hành hiện tại không nằm trong danh sách hỗ trợ chính thức." >&2
    exit 1
fi
echo "   ✔ Xác nhận hệ điều hành: Arch Linux tương thích."

# Verify build toolchain
for tool in cargo rustc; do
    if ! command -v "$tool" &>/dev/null; then
        echo "❌ Lỗi: Không tìm thấy '$tool'. Vui lòng cài đặt Rust toolchain trước khi tiếp tục." >&2
        exit 1
    fi
done
echo "   ✔ Rust toolchain: $(rustc --version) | $(cargo --version)"

# ------------------------------------------------------------------------------
# 2. Conflict Detection: fcitx5 / ibus
# ------------------------------------------------------------------------------
echo "🔍 [2/7] Kiểm tra xung đột bộ gõ (fcitx5 / ibus)..."
CONFLICTING_PIDS=()
if pgrep -x fcitx5 &>/dev/null; then
    while IFS= read -r pid; do
        [ -n "$pid" ] && CONFLICTING_PIDS+=("$pid")
    done < <(pgrep -x fcitx5)
fi
if pgrep -x fcitx &>/dev/null; then
    while IFS= read -r pid; do
        [ -n "$pid" ] && CONFLICTING_PIDS+=("$pid")
    done < <(pgrep -x fcitx)
fi
if pgrep -x ibus-daemon &>/dev/null; then
    while IFS= read -r pid; do
        [ -n "$pid" ] && CONFLICTING_PIDS+=("$pid")
    done < <(pgrep -x ibus-daemon)
fi
if pgrep -x ibus &>/dev/null; then
    while IFS= read -r pid; do
        [ -n "$pid" ] && CONFLICTING_PIDS+=("$pid")
    done < <(pgrep -x ibus)
fi

if [ ${#CONFLICTING_PIDS[@]} -gt 0 ]; then
    echo "   ⚠️  Phát hiện tiến trình bộ gõ khác đang chạy (PID: ${CONFLICTING_PIDS[*]})."
    if [ "$DRY_RUN" = true ]; then
        echo "   [DRY-RUN] Sẽ đóng các tiến trình fcitx5/ibus đang chạy."
    else
        TERMINATE_CONFLICTS=false
        if [ "$NON_INTERACTIVE" = true ] || [ "$KILL_CONFLICTS" = true ]; then
            TERMINATE_CONFLICTS=true
        else
            read -r -p "   Bạn có muốn đóng các tiến trình bộ gõ cũ để tránh xung đột không? [Y/n] " answer
            answer=${answer:-Y}
            if [[ "$answer" =~ ^[Yy]$ ]]; then
                TERMINATE_CONFLICTS=true
            fi
        fi

        if [ "$TERMINATE_CONFLICTS" = true ]; then
            echo "   Đang đóng tiến trình xung đột..."
            pkill -x fcitx5 2>/dev/null || true
            pkill -x fcitx 2>/dev/null || true
            pkill -x ibus-daemon 2>/dev/null || true
            pkill -x ibus 2>/dev/null || true
            sleep 0.5
            echo "   ✔ Đã dừng các tiến trình bộ gõ cũ."
        else
            echo "   ⚠️  Lưu ý: Chạy đồng thời nhiều bộ gõ có thể gây xung đột phím Wayland."
        fi
    fi
else
    echo "   ✔ Không phát hiện bộ gõ nào khác đang chạy."
fi

# Deactivate fcitx5 in Hyprland Startup_Apps if present
STARTUP_CONF="${HOME}/.config/hypr/UserConfigs/Startup_Apps.conf"
if [ -f "$STARTUP_CONF" ] && grep -qE "^exec-once = fcitx5" "$STARTUP_CONF"; then
    if [ "$DRY_RUN" = true ]; then
        echo "   [DRY-RUN] Sẽ tạm thời vô hiệu hóa 'exec-once = fcitx5' trong $STARTUP_CONF."
    else
        sed -i 's/^exec-once = fcitx5/# exec-once = fcitx5 # VietKeyra disabled/' "$STARTUP_CONF"
        echo "   ✔ Đã tạm thời vô hiệu hóa khởi động fcitx5 trong Startup_Apps.conf."
    fi
fi

# ------------------------------------------------------------------------------
# 3. Compile Workspace in Release Mode
# ------------------------------------------------------------------------------
echo "🔨 [3/7] Biên dịch Workspace VietKeyra (Release mode)..."
if [ "$DRY_RUN" = true ]; then
    echo "   [DRY-RUN] Sẽ thực thi: cargo build --release --workspace"
else
    cargo build --release --workspace
    for bin in vietkeyra-daemon vietkeyra-cli vietkeyra-gui; do
        if [ ! -f "${PROJECT_ROOT}/target/release/$bin" ]; then
            echo "❌ Lỗi: Biên dịch thành công nhưng không tìm thấy binary target/release/$bin" >&2
            exit 1
        fi
    done
    echo "   ✔ Biên dịch thành công 3 binary chính."
fi

# ------------------------------------------------------------------------------
# 4. Install Binaries to ~/.local/bin/
# ------------------------------------------------------------------------------
BIN_DIR="${HOME}/.local/bin"
echo "📦 [4/7] Cài đặt các binary vào $BIN_DIR..."
if [ "$DRY_RUN" = true ]; then
    echo "   [DRY-RUN] Sẽ cài đặt: vietkeyra-daemon, vietkeyra-cli, vietkeyra-gui vào $BIN_DIR"
    echo "   [DRY-RUN] Sẽ tạo symlink vietkeyra-ctl -> vietkeyra-cli"
else
    mkdir -p "$BIN_DIR"
    cp "${PROJECT_ROOT}/target/release/vietkeyra-daemon" "${BIN_DIR}/"
    cp "${PROJECT_ROOT}/target/release/vietkeyra-cli" "${BIN_DIR}/"
    cp "${PROJECT_ROOT}/target/release/vietkeyra-gui" "${BIN_DIR}/"
    chmod 755 "${BIN_DIR}/vietkeyra-daemon" "${BIN_DIR}/vietkeyra-cli" "${BIN_DIR}/vietkeyra-gui"
    ln -sf "vietkeyra-cli" "${BIN_DIR}/vietkeyra-ctl"
    echo "   ✔ Đã sao chép và phân quyền binary thành công."
fi

# ------------------------------------------------------------------------------
# 5. Backup Existing Configurations with Manifest
# ------------------------------------------------------------------------------
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BACKUP_BASE="${HOME}/.config/vietkeyra/backups"
BACKUP_DIR="${BACKUP_BASE}/${TIMESTAMP}"

echo "💾 [5/7] Sao lưu cấu hình hiện tại..."
CONFIG_FILES_TO_BACKUP=(
    "${HOME}/.config/hypr/UserConfigs/Startup_Apps.conf"
    "${HOME}/.config/hypr/UserConfigs/UserKeybinds.conf"
    "${HOME}/.config/hypr/hyprland.conf"
    "${HOME}/.config/waybar/config"
    "${HOME}/.config/waybar/UserModules"
    "${HOME}/.config/waybar/style.css"
)

if [ "$DRY_RUN" = true ]; then
    echo "   [DRY-RUN] Sẽ tạo bản sao lưu tại $BACKUP_DIR kèm manifest.json"
else
    mkdir -p "$BACKUP_DIR"
    MANIFEST_FILE="${BACKUP_DIR}/manifest.json"
    
    # Initialize JSON array
    MANIFEST_ENTRIES=()

    for cfg in "${CONFIG_FILES_TO_BACKUP[@]}"; do
        if [ -e "$cfg" ]; then
            # Resolve real target if symlink
            real_path=$(realpath "$cfg")
            base_name=$(basename "$cfg")
            # If symlink points elsewhere, preserve name uniqueness
            dest_file="${base_name}"
            if [ -f "${BACKUP_DIR}/${dest_file}" ]; then
                dest_file="${dest_file}_$(basename "$(dirname "$cfg")")"
            fi

            cp "$real_path" "${BACKUP_DIR}/${dest_file}"
            csum=$(sha256sum "$real_path" | awk '{print $1}')
            
            MANIFEST_ENTRIES+=("{\"original_path\": \"$cfg\", \"real_path\": \"$real_path\", \"backup_file\": \"$dest_file\", \"sha256\": \"$csum\"}")
        fi
    done

    # Write manifest.json
    {
        echo "{"
        echo "  \"timestamp\": \"$TIMESTAMP\","
        echo "  \"backup_dir\": \"$BACKUP_DIR\","
        echo "  \"files\": ["
        for i in "${!MANIFEST_ENTRIES[@]}"; do
            if [ "$i" -lt $(( ${#MANIFEST_ENTRIES[@]} - 1 )) ]; then
                echo "    ${MANIFEST_ENTRIES[$i]},"
            else
                echo "    ${MANIFEST_ENTRIES[$i]}"
            fi
        done
        echo "  ]"
        echo "}"
    } > "$MANIFEST_FILE"

    # Update latest symlink
    ln -sfn "$BACKUP_DIR" "${BACKUP_BASE}/latest"
    echo "   ✔ Đã sao lưu các cấu hình vào: $BACKUP_DIR"
    echo "   ✔ Đã tạo symlink: ${BACKUP_BASE}/latest"
fi

# ------------------------------------------------------------------------------
# 6. Inject Configurations with Sentinel Blocks
# ------------------------------------------------------------------------------
echo "⚙️  [6/7] Thiết lập cấu hình Hyprland và Waybar..."

# 6a. Hyprland Autostart
HYPR_AUTOSTART=""
if [ -f "${HOME}/.config/hypr/UserConfigs/Startup_Apps.conf" ]; then
    HYPR_AUTOSTART="${HOME}/.config/hypr/UserConfigs/Startup_Apps.conf"
elif [ -f "${HOME}/.config/hypr/hyprland.conf" ]; then
    HYPR_AUTOSTART="${HOME}/.config/hypr/hyprland.conf"
fi

if [ -n "$HYPR_AUTOSTART" ]; then
    if grep -q "VIETKEYRA START" "$HYPR_AUTOSTART"; then
        echo "   ℹ️  Khởi động tự động đã tồn tại trong $HYPR_AUTOSTART (bỏ qua)."
    else
        if [ "$DRY_RUN" = true ]; then
            echo "   [DRY-RUN] Sẽ thêm autostart vào $HYPR_AUTOSTART"
        else
            cat <<'EOF' >> "$HYPR_AUTOSTART"

# >>> VIETKEYRA START >>>
exec-once = vietkeyra-daemon
# <<< VIETKEYRA END <<<
EOF
            echo "   ✔ Đã thêm cấu hình khởi động vào $HYPR_AUTOSTART"
        fi
    fi
fi

# 6b. Hyprland Keybinds
HYPR_KEYBINDS=""
if [ -f "${HOME}/.config/hypr/UserConfigs/UserKeybinds.conf" ]; then
    HYPR_KEYBINDS="${HOME}/.config/hypr/UserConfigs/UserKeybinds.conf"
elif [ -f "${HOME}/.config/hypr/hyprland.conf" ]; then
    HYPR_KEYBINDS="${HOME}/.config/hypr/hyprland.conf"
fi

if [ -n "$HYPR_KEYBINDS" ]; then
    if grep -q "VIETKEYRA START" "$HYPR_KEYBINDS"; then
        echo "   ℹ️  Phím tắt đã tồn tại trong $HYPR_KEYBINDS (bỏ qua)."
    else
        if [ "$DRY_RUN" = true ]; then
            echo "   [DRY-RUN] Sẽ thêm phím tắt vào $HYPR_KEYBINDS"
        else
            cat <<'EOF' >> "$HYPR_KEYBINDS"

# >>> VIETKEYRA START >>>
bindd = SUPER, SPACE, Chuyển đổi bộ gõ VietKeyra, exec, vietkeyra-cli toggle
bindd = SUPER SHIFT, V, Mở Bảng điều khiển VietKeyra, exec, vietkeyra-gui
# <<< VIETKEYRA END <<<
EOF
            echo "   ✔ Đã thêm phím tắt (SUPER+SPACE, SUPER+SHIFT+V) vào $HYPR_KEYBINDS"
        fi
    fi
fi

# 6c. Waybar UserModules
WAYBAR_USER_MODULES="${HOME}/.config/waybar/UserModules"
if [ -f "$WAYBAR_USER_MODULES" ]; then
    if grep -q "VIETKEYRA START" "$WAYBAR_USER_MODULES"; then
        echo "   ℹ️  Module Waybar đã có trong $WAYBAR_USER_MODULES (bỏ qua)."
    else
        if [ "$DRY_RUN" = true ]; then
            echo "   [DRY-RUN] Sẽ cấu hình module 'custom/vietkeyra' vào $WAYBAR_USER_MODULES"
        else
            # Insert custom/vietkeyra into UserModules JSON
            python3 -c "
import re

path = '$WAYBAR_USER_MODULES'
with open(path, 'r', encoding='utf-8') as f:
    content = f.read()

module_block = '''// >>> VIETKEYRA START >>>
\"custom/vietkeyra\": {
    \"format\": \"{}\",
    \"exec\": \"vietkeyra-cli waybar\",
    \"return-type\": \"json\",
    \"signal\": 8,
    \"interval\": \"once\",
    \"on-click\": \"vietkeyra-cli toggle\",
    \"on-click-right\": \"vietkeyra-gui\",
    \"tooltip\": true
},
// <<< VIETKEYRA END <<<'''

# Insert inside the outer json braces
last_brace = content.rfind('}')
if last_brace != -1:
    new_content = content[:last_brace] + module_block + '\n' + content[last_brace:]
    with open(path, 'w', encoding='utf-8') as f:
        f.write(new_content)
"
            echo "   ✔ Đã đăng ký module custom/vietkeyra vào $WAYBAR_USER_MODULES"
        fi
    fi
fi

# 6d. Waybar Config modules-right
WAYBAR_CONFIG="${HOME}/.config/waybar/config"
if [ -e "$WAYBAR_CONFIG" ]; then
    REAL_WAYBAR_CONFIG=$(realpath "$WAYBAR_CONFIG")
    if ! grep -q "custom/vietkeyra" "$REAL_WAYBAR_CONFIG"; then
        if [ "$DRY_RUN" = true ]; then
            echo "   [DRY-RUN] Sẽ thêm 'custom/vietkeyra' vào modules-right của $REAL_WAYBAR_CONFIG"
        else
            # Insert custom/vietkeyra into modules-right array
            python3 -c "
path = '$REAL_WAYBAR_CONFIG'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

# Locate 'modules-right'
idx = text.find('\"modules-right\":')
if idx != -1:
    arr_start = text.find('[', idx)
    if arr_start != -1:
        insertion = '\n\t\"custom/vietkeyra\",'
        new_text = text[:arr_start+1] + insertion + text[arr_start+1:]
        with open(path, 'w', encoding='utf-8') as f:
            f.write(new_text)
"
            echo "   ✔ Đã đưa 'custom/vietkeyra' vào thanh hiển thị của Waybar."
        fi
    fi
fi

# 6e. Waybar Stylesheet
WAYBAR_CSS="${HOME}/.config/waybar/style.css"
if [ -e "$WAYBAR_CSS" ]; then
    REAL_WAYBAR_CSS=$(realpath "$WAYBAR_CSS")
    if grep -q "VIETKEYRA START" "$REAL_WAYBAR_CSS"; then
        echo "   ℹ️  CSS VietKeyra đã có trong $REAL_WAYBAR_CSS (bỏ qua)."
    else
        if [ "$DRY_RUN" = true ]; then
            echo "   [DRY-RUN] Sẽ thêm style #custom-vietkeyra vào $REAL_WAYBAR_CSS"
        else
            cat <<'EOF' >> "$REAL_WAYBAR_CSS"

/* >>> VIETKEYRA START >>> */
#custom-vietkeyra {
    padding: 0 10px;
    margin: 0 4px;
    border-radius: 12px;
    font-weight: bold;
    transition: all 0.2s ease-in-out;
}

#custom-vietkeyra.vi {
    color: #a6e3a1; /* Catppuccin Green / Emerald */
    background-color: rgba(166, 227, 161, 0.12);
    border: 1px solid rgba(166, 227, 161, 0.3);
}

#custom-vietkeyra.en {
    color: #89b4fa; /* Catppuccin Blue / Cyan */
    background-color: rgba(137, 180, 250, 0.12);
    border: 1px solid rgba(137, 180, 250, 0.3);
}

#custom-vietkeyra.disabled {
    color: #6c7086;
    background-color: rgba(108, 112, 134, 0.12);
    border: 1px solid rgba(108, 112, 134, 0.3);
}
/* <<< VIETKEYRA END <<< */
EOF
            echo "   ✔ Đã thêm định kiểu Catppuccin vào $REAL_WAYBAR_CSS"
        fi
    fi
fi

# ------------------------------------------------------------------------------
# 7. Install Desktop Entry
# ------------------------------------------------------------------------------
echo "🖥️  [7/7] Cài đặt Desktop Entry..."
DESKTOP_DIR="${HOME}/.local/share/applications"
if [ "$DRY_RUN" = true ]; then
    echo "   [DRY-RUN] Sẽ sao chép vietkeyra.desktop vào $DESKTOP_DIR"
else
    mkdir -p "$DESKTOP_DIR"
    cp "${PROJECT_ROOT}/configs/vietkeyra.desktop" "${DESKTOP_DIR}/"
    chmod 644 "${DESKTOP_DIR}/vietkeyra.desktop"
    if command -v update-desktop-database &>/dev/null; then
        update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
    fi
    echo "   ✔ Đã cài đặt menu ứng dụng: ${DESKTOP_DIR}/vietkeyra.desktop"
fi

# ------------------------------------------------------------------------------
# Completion Summary
# ------------------------------------------------------------------------------
echo ""
echo "=============================================================================="
if [ "$DRY_RUN" = true ]; then
    echo "🎉 [DRY-RUN] KIỂM TRA MÔ PHỎNG HOÀN TẤT: KHÔNG CÓ THAY ĐỔI NÀO ĐƯỢC GHI LÊN ĐĨA."
else
    echo "🎉 CÀI ĐẶT VIETKEYRA HOÀN TẤT THÀNH CÔNG!"
    echo "   - Binaries:   ~/.local/bin/{vietkeyra-daemon, vietkeyra-cli, vietkeyra-gui, vietkeyra-ctl}"
    echo "   - Backup:     $BACKUP_DIR"
    echo "   - Phím tắt:   Super + Space (Đổi EN/VI), Super + Shift + V (Bảng điều khiển)"
    echo "   - Để khởi động ngay bây giờ:"
    echo "       vietkeyra-daemon &"
    echo "       pkill -SIGUSR2 waybar  # Để nạp lại giao diện Waybar"
fi
echo "=============================================================================="
exit 0
