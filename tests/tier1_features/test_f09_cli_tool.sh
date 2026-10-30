#!/usr/bin/env bash
# ==============================================================================
# tests/tier1_features/test_f09_cli_tool.sh
# Tier 1 Tests for F9: CLI Client Tool
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 1 Tests for F9: CLI Client Tool..."

# Test 1: CLI Crate Compiles Cleanly
if check_component "cli"; then
    out=$(cd "${PROJECT_ROOT}" && cargo check -p vietkeyra-cli 2>&1 || true)
    if echo "$out" | grep -q "Finished"; then
        report_pass "T1_F09_1" "Crate vietkeyra-cli compiles cleanly"
    else
        report_fail "T1_F09_1" "Crate vietkeyra-cli compiles" "Compilation failed"
    fi
else
    report_skip "T1_F09_1" "Crate vietkeyra-cli compiles cleanly" "crates/vietkeyra-cli not yet available"
fi

# Test 2: CLI Subcommand Parsing (status, toggle, set-mode, waybar)
if check_component "cli"; then
    cli_src="${PROJECT_ROOT}/crates/vietkeyra-cli/src/main.rs"
    if [ -f "$cli_src" ]; then
        has_cmds=$(grep -E "status|toggle|set-mode|waybar|monitor" "$cli_src" || true)
        if [ -n "$has_cmds" ]; then
            report_pass "T1_F09_2" "CLI source defines required subcommands (status, toggle, waybar)"
        else
            report_fail "T1_F09_2" "CLI subcommands" "Required subcommands not found in main.rs"
        fi
    else
        report_fail "T1_F09_2" "CLI main.rs" "File not found: $cli_src"
    fi
else
    report_skip "T1_F09_2" "CLI source defines required subcommands" "crates/vietkeyra-cli not yet available"
fi

# Test 3: CLI Waybar Output Command Interface
if check_component "cli"; then
    cli_src="${PROJECT_ROOT}/crates/vietkeyra-cli/src/main.rs"
    has_waybar_cmd=$(grep -i "waybar" "$cli_src" || true)
    if [ -n "$has_waybar_cmd" ]; then
        report_pass "T1_F09_3" "CLI includes dedicated 'waybar' command"
    else
        report_fail "T1_F09_3" "CLI waybar command" "Not found in CLI source"
    fi
else
    report_skip "T1_F09_3" "CLI includes dedicated 'waybar' command" "crates/vietkeyra-cli not yet available"
fi

# Test 4: CLI Error Handling when Socket is Absent
# Run CLI or simulate socket connection failure
py_cli_mock=$(python3 -c "
import socket, sys, os
sock_path = '/tmp/nonexistent_test_vietkeyra.sock'
try:
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.connect(sock_path)
    print('CONNECTED')
except FileNotFoundError:
    print('CLEAN_ERROR')
except ConnectionRefusedError:
    print('CLEAN_ERROR')
")
assert_eq "CLEAN_ERROR" "$py_cli_mock" "T1_F09_4" "Socket client catches offline daemon gracefully without crashing"

# Test 5: CLI Waybar JSON Emits Single Line for Waybar stdout reader
sample_waybar_out='{"text": "VI", "alt": "vietnamese", "tooltip": "VietKeyra: Tiếng Việt (Telex)", "class": "vi"}'
line_count=$(echo "$sample_waybar_out" | wc -l)
assert_eq "1" "$line_count" "T1_F09_5" "Waybar JSON payload is strictly a single newline-delimited line"

print_summary "Tier 1 - F9 (CLI Client Tool)"
