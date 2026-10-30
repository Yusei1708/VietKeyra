#!/usr/bin/env bash
# ==============================================================================
# tests/tier1_features/test_f13_uninstaller.sh
# Tier 1 Tests for F13: Automated Uninstaller Script
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 1 Tests for F13: Automated Uninstaller Script..."

UNINSTALL_SH="${PROJECT_ROOT}/uninstall.sh"

# Test 1: uninstall.sh Syntax Check via bash -n
if check_component "uninstaller"; then
    if bash -n "$UNINSTALL_SH" 2>&1; then
        report_pass "T1_F13_1" "uninstall.sh syntax is valid (bash -n passes)"
    else
        report_fail "T1_F13_1" "uninstall.sh syntax" "bash -n reported syntax errors"
    fi
else
    report_skip "T1_F13_1" "uninstall.sh syntax is valid" "uninstall.sh not yet available"
fi

# Test 2: Process Termination of Running VietKeyra Daemons
if check_component "uninstaller"; then
    has_kill_logic=$(grep -E "pkill|killall" "$UNINSTALL_SH" | grep -E "vietkeyra" || true)
    if [ -n "$has_kill_logic" ]; then
        report_pass "T1_F13_2" "uninstall.sh terminates running vietkeyra processes"
    else
        report_fail "T1_F13_2" "Process termination" "pkill vietkeyra not found in uninstall.sh"
    fi
else
    report_skip "T1_F13_2" "uninstall.sh terminates running vietkeyra processes" "uninstall.sh not yet available"
fi

# Test 3: Binary Cleanup from ~/.local/bin/
if check_component "uninstaller"; then
    has_bin_cleanup=$(grep -E "rm.*\.local/bin/vietkeyra" "$UNINSTALL_SH" || true)
    if [ -n "$has_bin_cleanup" ]; then
        report_pass "T1_F13_3" "uninstall.sh cleans up installed binaries from ~/.local/bin"
    else
        report_fail "T1_F13_3" "Binary removal" "rm vietkeyra from ~/.local/bin not found"
    fi
else
    report_skip "T1_F13_3" "uninstall.sh cleans up installed binaries" "uninstall.sh not yet available"
fi

# Test 4: Marker Removal from Config Files (sed /d between markers)
if check_component "uninstaller"; then
    has_marker_strip=$(grep -E "VIETKEYRA START.*VIETKEYRA END|sed.*VIETKEYRA" "$UNINSTALL_SH" || true)
    if [ -n "$has_marker_strip" ]; then
        report_pass "T1_F13_4" "uninstall.sh strips marked blocks cleanly using sed"
    else
        report_fail "T1_F13_4" "Marker block stripping" "Marker strip command not found"
    fi
else
    report_skip "T1_F13_4" "uninstall.sh strips marked blocks cleanly" "uninstall.sh not yet available"
fi

# Test 5: Socket File Cleanup
if check_component "uninstaller"; then
    has_sock_clean=$(grep -E "rm.*vietkeyra\.sock|rm.*XDG_RUNTIME_DIR" "$UNINSTALL_SH" || true)
    if [ -n "$has_sock_clean" ]; then
        report_pass "T1_F13_5" "uninstall.sh unlinks stale socket files"
    else
        report_fail "T1_F13_5" "Socket cleanup" "Socket unlinking not found in uninstall.sh"
    fi
else
    report_skip "T1_F13_5" "uninstall.sh unlinks stale socket files" "uninstall.sh not yet available"
fi

print_summary "Tier 1 - F13 (Automated Uninstaller)"
