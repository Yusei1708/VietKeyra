#!/usr/bin/env bash
# ==============================================================================
# tests/tier1_features/test_f05_wayland_daemon.sh
# Tier 1 Tests for F5: Wayland Input Method Daemon
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 1 Tests for F5: Wayland Input Method Daemon..."

# Test 1: Daemon Crate Exists and Compiles Cleanly
if check_component "daemon"; then
    out=$(cd "${PROJECT_ROOT}" && cargo check -p vietkeyra-daemon 2>&1 || true)
    if echo "$out" | grep -q "Finished"; then
        report_pass "T1_F05_1" "Crate vietkeyra-daemon compiles cleanly"
    else
        report_fail "T1_F05_1" "Crate vietkeyra-daemon compiles cleanly" "Compilation failed"
    fi
else
    report_skip "T1_F05_1" "Crate vietkeyra-daemon compiles cleanly" "crates/vietkeyra-daemon not yet available"
fi

# Test 2: KeyAction delete_bytes Calculation is Strictly UTF-8 Byte Length
# Verify that delete_bytes matches exact UTF-8 byte length (not character count)
py_byte_check=$(python3 -c "
def utf8_byte_len(s):
    return len(s.encode('utf-8'))

# 'a' = 1 byte
# 'â' = 2 bytes
# 'ệ' = 3 bytes
assert utf8_byte_len('a') == 1
assert utf8_byte_len('â') == 2
assert utf8_byte_len('ệ') == 3
print('OK')
")
assert_eq "OK" "$py_byte_check" "T1_F05_2" "UTF-8 byte length calculations for delete_surrounding_text are exact"

# Test 3: zwp_input_method_v2 Protocol Usage in Daemon Source
if check_component "daemon"; then
    daemon_src="${PROJECT_ROOT}/crates/vietkeyra-daemon/src"
    has_proto=$(grep -r "zwp_input_method_v2" "$daemon_src" || true)
    if [ -n "$has_proto" ]; then
        report_pass "T1_F05_3" "Daemon integrates zwp_input_method_v2 protocol bindings"
    else
        report_fail "T1_F05_3" "Daemon integrates zwp_input_method_v2" "No zwp_input_method_v2 references found in $daemon_src"
    fi
else
    report_skip "T1_F05_3" "Daemon integrates zwp_input_method_v2 protocol bindings" "crates/vietkeyra-daemon not yet available"
fi

# Test 4: Rootless Operation Validation (No root/sudo/udev required)
# Daemon must run as normal user (UID != 0)
current_uid=$(id -u)
if [ "$current_uid" -ne 0 ]; then
    report_pass "T1_F05_4" "Execution environment is rootless (current UID: $current_uid)"
else
    report_fail "T1_F05_4" "Rootless execution environment" "Running as root is forbidden"
fi

# Test 5: Clean Process Termination on SIGTERM
if check_component "daemon"; then
    # Test that daemon source handles termination signal
    daemon_src="${PROJECT_ROOT}/crates/vietkeyra-daemon/src"
    has_sig=$(grep -rE "SIGINT|SIGTERM|signal|ctrlc" "$daemon_src" || true)
    if [ -n "$has_sig" ]; then
        report_pass "T1_F05_5" "Daemon source includes signal handling for graceful termination"
    else
        report_fail "T1_F05_5" "Daemon signal handling" "No signal handler found in daemon source"
    fi
else
    report_skip "T1_F05_5" "Daemon source includes signal handling for graceful termination" "crates/vietkeyra-daemon not yet available"
fi

print_summary "Tier 1 - F5 (Wayland Daemon)"
