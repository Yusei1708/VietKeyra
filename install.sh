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

echo "=== [3/6] Cài đặt Desktop Entry & Biểu tượng ứng dụng ==="
APP_DIR="${HOME}/.local/share/applications"
ICON_DIR="${HOME}/.local/share/icons/hicolor/128x128/apps"
SCALABLE_ICON_DIR="${HOME}/.local/share/icons/hicolor/scalable/apps"
mkdir -p "${APP_DIR}" "${ICON_DIR}" "${SCALABLE_ICON_DIR}" "${HOME}/.local/share/pixmaps" "${HOME}/.local/share/icons"

cp -f "${SCRIPT_DIR}/assets/icons/vietkeyra.png" "${ICON_DIR}/vietkeyra.png"
cp -f "${SCRIPT_DIR}/assets/icons/vietkeyra.png" "${HOME}/.local/share/pixmaps/vietkeyra.png"
cp -f "${SCRIPT_DIR}/assets/icons/vietkeyra.png" "${HOME}/.local/share/icons/vietkeyra.png"
cp -f "${SCRIPT_DIR}/assets/icons/vietkeyra.svg" "${SCALABLE_ICON_DIR}/vietkeyra.svg"

cat << EOF > "${APP_DIR}/vietkeyra.desktop"
[Desktop Entry]
Name=VietKeyra
GenericName=Bộ gõ tiếng Việt
Comment=Bộ gõ tiếng Việt độc lập 100% cho Linux (Wayland / Hyprland)
Exec=${BIN_DIR}/vietkeyra-ctl launch
Icon=vietkeyra
Terminal=false
Type=Application
Categories=Utility;
Keywords=VietKeyra;Unikey;Vietnamese;Telex;BoGo;IME;
StartupNotify=false
EOF
chmod +x "${APP_DIR}/vietkeyra.desktop"
update-desktop-database "${APP_DIR}" 2>/dev/null || true
gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" 2>/dev/null || true

echo "=== [4/6] Cấu hình Waybar ==="
# Cấu hình module trong UserModules
if [ -f "${WAYBAR_CONFIG_DIR}/UserModules" ]; then
    cat << 'EOF' > "${WAYBAR_CONFIG_DIR}/UserModules"
//* ---- VietKeyra Waybar Module ---- *//
{
"custom/vietkeyra": {
    "format": "  {}",
    "return-type": "json",
    "exec": "$HOME/.local/bin/vietkeyra-ctl waybar",
    "on-click": "$HOME/.local/bin/vietkeyra-ctl toggle",
    "on-click-right": "$HOME/.local/bin/vietkeyra-ctl quit",
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
	background: #0F1419;
	border-radius: 50px 50px;
	margin: 5px;
	padding: 2px 15px;
	font-weight: bold;
}
#custom-vietkeyra.vi {
	color: #39FF14;
}
#custom-vietkeyra.en {
	color: #B8CC52;
}
#custom-vietkeyra.offline {
	color: #F07178;
}
EOF
    fi
fi

echo "=== [5/6] Cấu hình tự khởi động cùng phiên Hyprland ==="
if [ -f "${HYPR_CONFIG_DIR}/Startup_Apps.conf" ]; then
    # Tắt fcitx5 để tránh xung đột cổng zwp_input_method_v2
    sed -i 's/^exec-once = fcitx5/#exec-once = fcitx5 # [vietkeyra-replaced]/' "${HYPR_CONFIG_DIR}/Startup_Apps.conf"
    if ! grep -q 'exec-once = vietkeyrad' "${HYPR_CONFIG_DIR}/Startup_Apps.conf"; then
        echo "exec-once = vietkeyrad" >> "${HYPR_CONFIG_DIR}/Startup_Apps.conf"
    fi
fi

echo "=== [6/6] Khởi động VietKeyra Daemon & cập nhật Waybar ==="
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
echo "  - Tìm kiếm ứng dụng: 'VietKeyra' trong trình tìm kiếm app."
echo "  - Chuột trái trên Waybar: Chuyển đổi qua lại EN / VI."
echo "  - Chuột phải trên Waybar: Thoát VietKeyra."
echo "=========================================================="
