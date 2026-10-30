#!/usr/bin/env bash
# ==============================================================================
# VietKeyra Uninstaller for Arch Linux + Hyprland + Waybar
# ==============================================================================
set -e

BIN_DIR="${HOME}/.local/bin"
WAYBAR_CONFIG_DIR="${HOME}/.config/waybar"
HYPR_CONFIG_DIR="${HOME}/.config/hypr/UserConfigs"

echo "=== [1/4] Dừng tiến trình VietKeyra ==="
pkill -f vietkeyrad 2>/dev/null || true
rm -f "${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/vietkeyra.sock"

echo "=== [2/5] Xóa tệp nhị phân, Desktop Entry & Biểu tượng ==="
rm -f "${BIN_DIR}/vietkeyrad" "${BIN_DIR}/vietkeyra-ctl"
rm -f "${HOME}/.local/share/applications/vietkeyra.desktop"
rm -f "${HOME}/.local/share/icons/hicolor/128x128/apps/vietkeyra.png"
rm -f "${HOME}/.local/share/icons/hicolor/scalable/apps/vietkeyra.svg"
rm -f "${HOME}/.local/share/icons/vietkeyra.png"
rm -f "${HOME}/.local/share/pixmaps/vietkeyra.png"
update-desktop-database "${HOME}/.local/share/applications" 2>/dev/null || true
gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" 2>/dev/null || true

echo "=== [3/5] Gỡ cấu hình Waybar ==="
REAL_WAYBAR_CONFIG=$(readlink -f "${WAYBAR_CONFIG_DIR}/config" 2>/dev/null || echo "${WAYBAR_CONFIG_DIR}/config")
if [ -f "${REAL_WAYBAR_CONFIG}" ]; then
    sed -i '/"custom\/vietkeyra",/d' "${REAL_WAYBAR_CONFIG}"
fi

if [ -f "${WAYBAR_CONFIG_DIR}/UserModules" ]; then
    cat << 'EOF' > "${WAYBAR_CONFIG_DIR}/UserModules"
//* ---- 💫 https://github.com/JaKooLit 💫 ---- *//
/* Waybar Modules Extras */

{

}
EOF
fi

REAL_WAYBAR_STYLE=$(readlink -f "${WAYBAR_CONFIG_DIR}/style.css" 2>/dev/null || echo "${WAYBAR_CONFIG_DIR}/style.css")
if [ -f "${REAL_WAYBAR_STYLE}" ]; then
    sed -i '/\/\* VietKeyra Status Indicator \*\//,+13d' "${REAL_WAYBAR_STYLE}" 2>/dev/null || true
fi

echo "=== [4/5] Khôi phục cấu hình khởi động Hyprland ==="
if [ -f "${HYPR_CONFIG_DIR}/Startup_Apps.conf" ]; then
    sed -i '/exec-once = vietkeyrad/d' "${HYPR_CONFIG_DIR}/Startup_Apps.conf"
    sed -i 's/#exec-once = fcitx5 # \[vietkeyra-replaced\]/exec-once = fcitx5/' "${HYPR_CONFIG_DIR}/Startup_Apps.conf"
fi

# Tải lại cấu hình Waybar
pkill -SIGUSR2 waybar 2>/dev/null || true

echo "=========================================================="
echo "  VietKeyra đã được gỡ cài đặt và dọn dẹp sạch sẽ!"
echo "=========================================================="
