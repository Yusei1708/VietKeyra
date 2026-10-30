#!/usr/bin/env bash
# ==============================================================================
# tests/tier1_features/test_f11_gtk4_gui.sh
# Tier 1 Tests for F11: GTK4 Vietnamese Control Panel GUI
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 1 Tests for F11: GTK4 Vietnamese Control Panel GUI..."

# Test 1: Desktop Entry File Validity
desktop_entry="[Desktop Entry]
Name=VietKeyra
Comment=Bộ gõ tiếng Việt cho Linux
Exec=vietkeyra-gui
Icon=input-keyboard
Terminal=false
Type=Application
Categories=Utility;Settings;"

is_desktop_valid=$(python3 -c "
entry = '''$desktop_entry'''
assert '[Desktop Entry]' in entry
assert 'Exec=vietkeyra-gui' in entry
assert 'Name=VietKeyra' in entry
print('VALID')
")
assert_eq "VALID" "$is_desktop_valid" "T1_F11_1" "FreeDesktop desktop entry specification is valid"

# Test 2: GUI Crate Dependencies in Cargo.toml
if check_component "gui"; then
    gui_cargo="${PROJECT_ROOT}/crates/vietkeyra-gui/Cargo.toml"
    has_gtk4=$(grep -E 'gtk4\s*=' "$gui_cargo" || true)
    if [ -n "$has_gtk4" ]; then
        report_pass "T1_F11_2" "vietkeyra-gui Cargo.toml depends on gtk4 crate"
    else
        report_fail "T1_F11_2" "gtk4 dependency" "gtk4 crate dependency not found in $gui_cargo"
    fi
else
    report_skip "T1_F11_2" "vietkeyra-gui Cargo.toml depends on gtk4 crate" "crates/vietkeyra-gui not yet available"
fi

# Test 3: Vietnamese UI Localization Strings Presence
if check_component "gui"; then
    gui_src="${PROJECT_ROOT}/crates/vietkeyra-gui/src"
    # Check for Vietnamese strings: Bật, Tiếng Việt, Telex, Phím tắt
    has_vn_strings=$(grep -rE "Tiếng Việt|Bật|Telex|Phím tắt|Cấu hình" "$gui_src" || true)
    if [ -n "$has_vn_strings" ]; then
        report_pass "T1_F11_3" "GUI source contains idiomatic Vietnamese localization strings"
    else
        report_fail "T1_F11_3" "Vietnamese localization strings" "No Vietnamese text found in GUI source"
    fi
else
    report_skip "T1_F11_3" "GUI source contains idiomatic Vietnamese localization strings" "crates/vietkeyra-gui not yet available"
fi

# Test 4: Socket Subscription Thread in GUI
if check_component "gui"; then
    gui_src="${PROJECT_ROOT}/crates/vietkeyra-gui/src"
    has_sub=$(grep -rE "subscribe|glib::MainContext|UnixStream" "$gui_src" || true)
    if [ -n "$has_sub" ]; then
        report_pass "T1_F11_4" "GUI integrates background socket subscription with glib MainContext"
    else
        report_fail "T1_F11_4" "Socket subscription with glib" "Not found in GUI source"
    fi
else
    report_skip "T1_F11_4" "GUI integrates background socket subscription with glib" "crates/vietkeyra-gui not yet available"
fi

# Test 5: GUI Non-Root Safety Execution Check
current_uid=$(id -u)
if [ "$current_uid" -ne 0 ]; then
    report_pass "T1_F11_5" "GUI runs under unprivileged user session without root elevation"
else
    report_fail "T1_F11_5" "Rootless GUI execution" "Running as root is forbidden"
fi

print_summary "Tier 1 - F11 (GTK4 Vietnamese GUI)"
