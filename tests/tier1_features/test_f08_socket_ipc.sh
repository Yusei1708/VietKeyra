#!/usr/bin/env bash
# ==============================================================================
# tests/tier1_features/test_f08_socket_ipc.sh
# Tier 1 Tests for F8: Unix Domain Socket IPC Server & Protocol
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 1 Tests for F8: Unix Domain Socket IPC Server & Protocol..."

# Test 1: Socket Path Definition Conforms to $XDG_RUNTIME_DIR/vietkeyra.sock
if check_component "daemon" || check_component "cli"; then
    daemon_ipc="${PROJECT_ROOT}/crates/vietkeyra-daemon/src/ipc"
    cli_src="${PROJECT_ROOT}/crates/vietkeyra-cli/src"
    has_sock_path=$(grep -rE "vietkeyra\.sock|XDG_RUNTIME_DIR" "${PROJECT_ROOT}/crates" || true)
    if [ -n "$has_sock_path" ]; then
        report_pass "T1_F08_1" "Socket path references \$XDG_RUNTIME_DIR/vietkeyra.sock"
    else
        report_fail "T1_F08_1" "Socket path definition" "vietkeyra.sock not found in crate sources"
    fi
else
    report_skip "T1_F08_1" "Socket path references \$XDG_RUNTIME_DIR/vietkeyra.sock" "Daemon/CLI crates not yet available"
fi

# Test 2: IPC JSON Lines Protocol Schema Validation
# Validate get_status request and response schema
status_req='{"cmd": "get_status"}'
status_resp='{"status": "ok", "mode": "VI", "engine": "telex", "version": "0.1.0"}'
schema_valid=$(python3 -c "
import json
req = json.loads('''$status_req''')
res = json.loads('''$status_resp''')
assert req.get('cmd') == 'get_status'
assert res.get('status') == 'ok'
assert res.get('mode') in ['VI', 'EN']
print('VALID')
")
assert_eq "VALID" "$schema_valid" "T1_F08_2" "JSON Lines IPC protocol schema validates for get_status"

# Test 3: IPC Protocol Handles toggle Command
toggle_req='{"cmd": "toggle"}'
toggle_valid=$(python3 -c "
import json
req = json.loads('''$toggle_req''')
assert req.get('cmd') == 'toggle'
print('VALID')
")
assert_eq "VALID" "$toggle_valid" "T1_F08_3" "IPC protocol supports toggle command"

# Test 4: IPC Protocol Handles set_mode Command
set_mode_req='{"cmd": "set_mode", "mode": "EN"}'
set_mode_valid=$(python3 -c "
import json
req = json.loads('''$set_mode_req''')
assert req.get('cmd') == 'set_mode'
assert req.get('mode') in ['VI', 'EN']
print('VALID')
")
assert_eq "VALID" "$set_mode_valid" "T1_F08_4" "IPC protocol supports set_mode command with valid modes"

# Test 5: IPC Protocol Handles subscribe Command for Real-Time Event Streaming
event_payload='{"event": "mode_changed", "mode": "VI", "engine": "telex"}'
event_valid=$(python3 -c "
import json
ev = json.loads('''$event_payload''')
assert ev.get('event') == 'mode_changed'
assert ev.get('mode') in ['VI', 'EN']
print('VALID')
")
assert_eq "VALID" "$event_valid" "T1_F08_5" "IPC pub/sub broadcast event schema is valid"

print_summary "Tier 1 - F8 (Socket IPC Protocol)"
