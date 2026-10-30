#!/usr/bin/env bash
# ==============================================================================
# uninstall.sh
# Automated Uninstaller for VietKeyra: Cleanly reverts all system modifications,
# removes binaries, and restores configs to a 100% pristine state.
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"

DRY_RUN=false
NON_INTERACTIVE=false
RESTORE_BACKUP=false

print_usage() {
    cat <<EOF
VietKeyra Uninstaller - Trình gỡ cài đặt Bộ gõ Tiếng Việt

Sử dụng:
  ./uninstall.sh [TÙY CHỌN]

Tùy chọn:
  --dry-run              Mô phỏng quy trình gỡ bỏ mà không xóa file hay sửa cấu hình
  -y, --yes              Chế độ tự động không hỏi xác nhận
  --restore-backup       Khôi phục toàn bộ các file từ bản sao lưu gần nhất (manifest.json)
  -h, --help             Hiển thị hướng dẫn sử dụng này

Ví dụ:
  ./uninstall.sh --dry-run
  ./uninstall.sh -y
  ./uninstall.sh --restore-backup
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
        --restore-backup)
            RESTORE_BACKUP=true
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
echo "          🧹 GỠ BỎ VIETKEYRA & HOÀN TÁC CẤU HÌNH HỆ THỐNG 🧹"
echo "=============================================================================="

# ------------------------------------------------------------------------------
# 1. Terminate Running VietKeyra Processes
# ------------------------------------------------------------------------------
echo "🛑 [1/5] Dừng các tiến trình VietKeyra đang hoạt động..."
if [ "$DRY_RUN" = true ]; then
    echo "   [DRY-RUN] Sẽ thực thi: pkill -x vietkeyra-daemon, pkill -x vietkeyra-gui"
else
    pkill -x vietkeyra-daemon 2>/dev/null || true
    pkill -x vietkeyra-gui 2>/dev/null || true
    echo "   ✔ Đã dừng toàn bộ tiến trình vietkeyra-daemon và vietkeyra-gui."
fi

# ------------------------------------------------------------------------------
# 2. Socket Cleanup
# ------------------------------------------------------------------------------
echo "🔌 [2/5] Dọn dẹp Unix socket IPC..."
RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
SOCKET_FILE="${RUNTIME_DIR}/vietkeyra.sock"
if [ "$DRY_RUN" = true ]; then
    echo "   [DRY-RUN] Sẽ xóa socket file nếu tồn tại: $SOCKET_FILE"
else
    rm -f "${SOCKET_FILE}"
    rm -f "${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/vietkeyra.sock"
    echo "   ✔ Đã dọn dẹp socket kết nối."
fi

# ------------------------------------------------------------------------------
# 3. Binary & Desktop Entry Removal
# ------------------------------------------------------------------------------
echo "🗑️  [3/5] Gỡ bỏ các binary và ứng dụng desktop..."
BIN_DIR="${HOME}/.local/bin"
DESKTOP_FILE="${HOME}/.local/share/applications/vietkeyra.desktop"

if [ "$DRY_RUN" = true ]; then
    echo "   [DRY-RUN] Sẽ xóa các binary trong $BIN_DIR: vietkeyra-daemon, vietkeyra-cli, vietkeyra-gui, vietkeyra-ctl"
    echo "   [DRY-RUN] Sẽ xóa desktop entry: $DESKTOP_FILE"
else
    rm -f "${HOME}/.local/bin/vietkeyra-daemon" "${HOME}/.local/bin/vietkeyra-cli" "${HOME}/.local/bin/vietkeyra-gui" "${HOME}/.local/bin/vietkeyra-ctl"
    rm -f ~/.local/bin/vietkeyra-daemon ~/.local/bin/vietkeyra-cli ~/.local/bin/vietkeyra-gui ~/.local/bin/vietkeyra-ctl
    rm -f "$DESKTOP_FILE"
    if command -v update-desktop-database &>/dev/null; then
        update-desktop-database "${HOME}/.local/share/applications" 2>/dev/null || true
    fi
    echo "   ✔ Đã xóa toàn bộ binary và desktop entry."
fi

# ------------------------------------------------------------------------------
# 4. Configuration Clean-up / Restoration
# ------------------------------------------------------------------------------
echo "📝 [4/5] Hoàn tác các khối cấu hình (Sentinel Marker Stripping)..."

BACKUP_LATEST="${HOME}/.config/vietkeyra/backups/latest"
MANIFEST="${BACKUP_LATEST}/manifest.json"

if [ "$RESTORE_BACKUP" = true ] && [ -f "$MANIFEST" ]; then
    echo "   🔄 Khôi phục trực tiếp từ bản sao lưu gần nhất: $BACKUP_LATEST"
    if [ "$DRY_RUN" = true ]; then
        echo "   [DRY-RUN] Sẽ khôi phục các file được liệt kê trong $MANIFEST"
    else
        python3 -c "
import json, os, shutil

manifest_path = '$MANIFEST'
try:
    with open(manifest_path, 'r', encoding='utf-8') as f:
        data = json.load(f)
    backup_dir = data.get('backup_dir', '')
    for item in data.get('files', []):
        target = os.path.expanduser(item.get('original_path', ''))
        real_target = item.get('real_path', target)
        src = os.path.join(backup_dir, item.get('backup_file', ''))
        if os.path.isfile(src):
            dest = real_target if os.path.exists(real_target) else target
            os.makedirs(os.path.dirname(dest), exist_ok=True)
            shutil.copy2(src, dest)
            print(f'   ✔ Đã khôi phục: {dest}')
except Exception as e:
    print(f'   ⚠️ Lỗi khôi phục backup: {e}')
"
    fi
else
    # Non-destructive Sentinel Marker removal using sed
    TARGET_CONFIG_FILES=(
        "${HOME}/.config/hypr/UserConfigs/Startup_Apps.conf"
        "${HOME}/.config/hypr/UserConfigs/UserKeybinds.conf"
        "${HOME}/.config/hypr/hyprland.conf"
        "${HOME}/.config/waybar/UserModules"
        "${HOME}/.config/waybar/style.css"
    )

    for cfg in "${TARGET_CONFIG_FILES[@]}"; do
        if [ -e "$cfg" ]; then
            real_cfg=$(realpath "$cfg")
            if grep -q "VIETKEYRA START" "$real_cfg" 2>/dev/null; then
                if [ "$DRY_RUN" = true ]; then
                    echo "   [DRY-RUN] Sẽ xóa khối marker VIETKEYRA START...VIETKEYRA END trong $cfg"
                else
                    sed -i '/VIETKEYRA START/,/VIETKEYRA END/d' "$real_cfg"
                    echo "   ✔ Đã xóa khối đánh dấu khỏi: $cfg"
                fi
            fi
        fi
    done

    # Remove custom/vietkeyra from active Waybar config if present
    WAYBAR_CONFIG="${HOME}/.config/waybar/config"
    if [ -e "$WAYBAR_CONFIG" ]; then
        REAL_WAYBAR_CONFIG=$(realpath "$WAYBAR_CONFIG")
        if grep -q "custom/vietkeyra" "$REAL_WAYBAR_CONFIG" 2>/dev/null; then
            if [ "$DRY_RUN" = true ]; then
                echo "   [DRY-RUN] Sẽ gỡ 'custom/vietkeyra' khỏi $REAL_WAYBAR_CONFIG"
            else
                python3 -c "
import re
path = '$REAL_WAYBAR_CONFIG'
try:
    with open(path, 'r', encoding='utf-8') as f:
        text = f.read()
    # Remove custom/vietkeyra from modules-right
    new_text = re.sub(r'\"custom/vietkeyra\",?\s*', '', text)
    with open(path, 'w', encoding='utf-8') as f:
        f.write(new_text)
except Exception as e:
    pass
"
                echo "   ✔ Đã gỡ 'custom/vietkeyra' khỏi danh sách module Waybar."
            fi
        fi
    fi

    # Restore commented out fcitx5 in Startup_Apps.conf if previously disabled by VietKeyra
    STARTUP_CONF="${HOME}/.config/hypr/UserConfigs/Startup_Apps.conf"
    if [ -f "$STARTUP_CONF" ] && grep -q "# exec-once = fcitx5 # VietKeyra disabled" "$STARTUP_CONF"; then
        if [ "$DRY_RUN" = true ]; then
            echo "   [DRY-RUN] Sẽ khôi phục dòng 'exec-once = fcitx5' trong $STARTUP_CONF"
        else
            sed -i 's/# exec-once = fcitx5 # VietKeyra disabled/exec-once = fcitx5/' "$STARTUP_CONF"
            echo "   ✔ Đã khôi phục dòng khởi động fcitx5 trong Startup_Apps.conf."
        fi
    fi
fi

# ------------------------------------------------------------------------------
# 5. Reload Compositor & Status Bar
# ------------------------------------------------------------------------------
echo "🔄 [5/5] Nạp lại trạng thái Hyprland và Waybar..."
if [ "$DRY_RUN" = true ]; then
    echo "   [DRY-RUN] Sẽ nạp lại cấu hình: hyprctl reload, pkill -SIGUSR2 waybar"
else
    command -v hyprctl &>/dev/null && hyprctl reload 2>/dev/null || true
    pkill -SIGUSR2 waybar 2>/dev/null || true
    echo "   ✔ Đã nạp lại cấu hình hệ thống."
fi

echo ""
echo "=============================================================================="
if [ "$DRY_RUN" = true ]; then
    echo "🎉 [DRY-RUN] KIỂM TRA MÔ PHỎNG GỠ CÀI ĐẶT HOÀN TẤT: KHÔNG CÓ THAY ĐỔI NÀO ĐƯỢC GHI LÊN ĐĨA."
else
    echo "🎉 GỠ BỎ VIETKEYRA HOÀN TẤT THÀNH CÔNG!"
    echo "   Hệ thống đã được trả về trạng thái sạch sẽ 100%."
fi
echo "=============================================================================="
exit 0
