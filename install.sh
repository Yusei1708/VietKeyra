#!/usr/bin/env bash
# ==============================================================================
# VietKeyra Installer for Arch Linux + Hyprland + Waybar
# ==============================================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="${HOME}/.local/bin"
WAYBAR_CONFIG_DIR="${HOME}/.config/waybar"
HYPR_CONFIG_DIR="${HOME}/.config/hypr/UserConfigs"

echo "=== [1/5] Biên dịch VietKeyra (Release) ==="
cd "${SCRIPT_DIR}"
cargo build --release

echo "=== [2/5] Cài đặt nhị phân vào ${BIN_DIR} ==="
mkdir -p "${BIN_DIR}"
cp -f "${SCRIPT_DIR}/target/release/vietkeyrad" "${BIN_DIR}/vietkeyrad"
cp -f "${SCRIPT_DIR}/target/release/vietkeyra-ctl" "${BIN_DIR}/vietkeyra-ctl"
chmod +x "${BIN_DIR}/vietkeyrad" "${BIN_DIR}/vietkeyra-ctl"

echo "=== [3/5] Cấu hình Waybar ==="
# Cấu hình module trong UserModules
if [ -f "${WAYBAR_CONFIG_DIR}/UserModules" ]; then
    cat << 'EOF' > "${WAYBAR_CONFIG_DIR}/UserModules"
//* ---- VietKeyra Waybar Module ---- *//
{
"custom/vietkeyra": {
    "format": "{}",
    "exec": "vietkeyra-ctl waybar",
    "on-click": "vietkeyra-ctl toggle",
    "restart-interval": 1,
    "signal": 8,
    "tooltip": true
}
}
EOF
fi

# Thêm "custom/vietkeyra" vào config chính của Waybar nếu chưa có
REAL_WAYBAR_CONFIG=$(readlink -f "${WAYBAR_CONFIG_DIR}/config" 2>/dev/null || echo "${WAYBAR_CONFIG_DIR}/config")
if [ -f "${REAL_WAYBAR_CONFIG}" ]; then
    if ! grep -q '"custom/vietkeyra"' "${REAL_WAYBAR_CONFIG}"; then
        sed -i '/"modules-right": \[/a \\t"custom/vietkeyra",' "${REAL_WAYBAR_CONFIG}"
    fi
fi

# Thêm CSS style cho icon VI/EN trên Waybar nếu chưa có
REAL_WAYBAR_STYLE=$(readlink -f "${WAYBAR_CONFIG_DIR}/style.css" 2>/dev/null || echo "${WAYBAR_CONFIG_DIR}/style.css")
if [ -f "${REAL_WAYBAR_STYLE}" ]; then
    if ! grep -q '#custom-vietkeyra' "${REAL_WAYBAR_STYLE}"; then
        cat << 'EOF' >> "${REAL_WAYBAR_STYLE}"

/* VietKeyra Status Indicator */
#custom-vietkeyra {
    padding: 0 10px;
    margin: 3px 2px;
    border-radius: 6px;
    font-weight: bold;
}
#custom-vietkeyra.vi {
    color: #a6e3a1;
}
#custom-vietkeyra.en {
    color: #cdd6f4;
}
EOF
    fi
fi

echo "=== [4/5] Cấu hình tự khởi động cùng phiên Hyprland ==="
if [ -f "${HYPR_CONFIG_DIR}/Startup_Apps.conf" ]; then
    # Tắt fcitx5 để tránh xung đột cổng zwp_input_method_v2
    sed -i 's/^exec-once = fcitx5/#exec-once = fcitx5 # [vietkeyra-replaced]/' "${HYPR_CONFIG_DIR}/Startup_Apps.conf"
    if ! grep -q 'exec-once = vietkeyrad' "${HYPR_CONFIG_DIR}/Startup_Apps.conf"; then
        echo "exec-once = vietkeyrad" >> "${HYPR_CONFIG_DIR}/Startup_Apps.conf"
    fi
fi

echo "=== [5/5] Khởi động VietKeyra Daemon & cập nhật Waybar ==="
# Tắt fcitx5 nếu đang chạy để nhường quyền IME
pkill -f fcitx5 2>/dev/null || true
# Dừng daemon cũ nếu có
pkill -f vietkeyrad 2>/dev/null || true
sleep 0.3

# Khởi động vietkeyrad mới
"${BIN_DIR}/vietkeyrad" >/dev/null 2>&1 &
sleep 0.3

# Tải lại cấu hình Waybar
pkill -SIGUSR2 waybar 2>/dev/null || true
pkill -RTMIN+8 waybar 2>/dev/null || true

echo "=========================================================="
echo "  VietKeyra đã được cài đặt và khởi động thành công!"
echo "  - Trạng thái mặc định: TIẾNG VIỆT (VI, Telex truyền thống)."
echo "  - Chuyển đổi EN / VI: Nhấp chuột vào biểu tượng trên Waybar."
echo "=========================================================="
