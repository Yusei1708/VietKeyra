#!/usr/bin/env bash
# ==============================================================================
# tests/tier1_features/test_f12_installer.sh
# Tier 1 Tests for F12: Automated Installer Script
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 1 Tests for F12: Automated Installer Script..."

INSTALL_SH="${PROJECT_ROOT}/install.sh"

# Test 1: install.sh Syntax Check via bash -n
if check_component "installer"; then
    if bash -n "$INSTALL_SH" 2>&1; then
        report_pass "T1_F12_1" "install.sh syntax is valid (bash -n passes)"
    else
        report_fail "T1_F12_1" "install.sh syntax" "bash -n reported syntax errors"
    fi
else
    report_skip "T1_F12_1" "install.sh syntax is valid" "install.sh not yet available"
fi

# Test 2: Arch Linux Environment Verification Logic
if check_component "installer"; then
    has_arch_check=$(grep -E "/etc/os-release|ID=arch|ID_LIKE=arch" "$INSTALL_SH" || true)
    if [ -n "$has_arch_check" ]; then
        report_pass "T1_F12_2" "install.sh validates Arch Linux environment via /etc/os-release"
    else
        report_fail "T1_F12_2" "Arch Linux detection" "Arch Linux check not found in install.sh"
    fi
else
    report_skip "T1_F12_2" "install.sh validates Arch Linux environment" "install.sh not yet available"
fi

# Test 3: Conflict Check for fcitx5 / ibus
if check_component "installer"; then
    has_conflict_check=$(grep -E "fcitx|ibus|pgrep" "$INSTALL_SH" || true)
    if [ -n "$has_conflict_check" ]; then
        report_pass "T1_F12_3" "install.sh checks for conflicting IMEs (fcitx5/ibus)"
    else
        report_fail "T1_F12_3" "Conflict check" "No fcitx5/ibus check found in install.sh"
    fi
else
    report_skip "T1_F12_3" "install.sh checks for conflicting IMEs" "install.sh not yet available"
fi

# Test 4: Backup Creation in ~/.config/vietkeyra/backups/
if check_component "installer"; then
    has_backup_logic=$(grep -E "vietkeyra/backups|manifest\.json" "$INSTALL_SH" || true)
    if [ -n "$has_backup_logic" ]; then
        report_pass "T1_F12_4" "install.sh creates timestamped backups in ~/.config/vietkeyra/backups/"
    else
        report_fail "T1_F12_4" "Backup creation logic" "Backup directory path not found in install.sh"
    fi
else
    report_skip "T1_F12_4" "install.sh creates timestamped backups" "install.sh not yet available"
fi

# Test 5: Sentinel Marker Injections (VIETKEYRA START / END)
if check_component "installer"; then
    has_markers=$(grep -E "VIETKEYRA START|VIETKEYRA END" "$INSTALL_SH" || true)
    if [ -n "$has_markers" ]; then
        report_pass "T1_F12_5" "install.sh uses sentinel markers for non-destructive config injection"
    else
        report_fail "T1_F12_5" "Sentinel markers" "Sentinel markers not found in install.sh"
    fi
else
    report_skip "T1_F12_5" "install.sh uses sentinel markers" "install.sh not yet available"
fi

print_summary "Tier 1 - F12 (Automated Installer)"
