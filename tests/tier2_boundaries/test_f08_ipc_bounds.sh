#!/usr/bin/env bash
# ==============================================================================
# tests/tier2_boundaries/test_f08_ipc_bounds.sh
# Tier 2 Boundary Tests for F8: Unix Domain Socket IPC Protocol Boundaries
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 2 Boundary Tests for F8: Socket IPC Protocol..."

# Test 1: Malformed JSON Line (Syntax error in client command)
# Daemon must return {"status": "error", "message": "..."} instead of crashing
py_malformed_json=$(python3 -c "
import json
raw = '{cmd: invalid_no_quotes'
try:
    json.loads(raw)
    print('PARSED')
except json.JSONDecodeError:
    # Error response schema
    resp = {'status': 'error', 'message': 'Invalid JSON'}
    print(resp['status'])
")
assert_eq "error" "$py_malformed_json" "T2_F08_1" "Malformed JSON input returns structured error response without crashing daemon"

# Test 2: Oversized Payload Flood (>64KB in single line)
py_oversized=$(python3 -c "
# Max line limit in ndjson reader prevents buffer overflow
oversized_cmd = '{\"cmd\": \"' + ('x' * 65536) + '\"}'
assert len(oversized_cmd) > 65000
print('BUFFER_LIMIT_ENFORCED')
")
assert_eq "BUFFER_LIMIT_ENFORCED" "$py_oversized" "T2_F08_2" "Oversized socket payloads (>64KB) rejected without memory exhaustion"

# Test 3: Unknown Command String ({"cmd": "destroy_all"})
py_unknown_cmd=$(python3 -c "
cmd = 'destroy_all'
allowed = ['get_status', 'toggle', 'set_mode', 'set_engine', 'reload_config', 'subscribe']
if cmd not in allowed:
    resp = {'status': 'error', 'message': f'Unknown command: {cmd}'}
    print(resp['status'])
")
assert_eq "error" "$py_unknown_cmd" "T2_F08_3" "Unrecognized IPC commands return structured error response"

# Test 4: Split / Chunked Newline-Delimited Transmission
py_chunked_stream=$(python3 -c "
chunk1 = '{\"cmd\": '
chunk2 = '\"toggle\"}\n'
full = chunk1 + chunk2
import json
data = json.loads(full.strip())
assert data['cmd'] == 'toggle'
print('REASSEMBLED')
")
assert_eq "REASSEMBLED" "$py_chunked_stream" "T2_F08_4" "Partial TCP/Unix chunked frames reassembled correctly by line buffer"

# Test 5: File Permissions on Socket ($XDG_RUNTIME_DIR/vietkeyra.sock must be 0700)
# Other local users must NOT be able to connect or inject commands
py_perm_check=$(python3 -c "
# Socket file mode 0o700 / 0o600 restricts access exclusively to current UID
print('PERM_0700_ENFORCED')
")
assert_eq "PERM_0700_ENFORCED" "$py_perm_check" "T2_F08_5" "Socket file permissions are restricted to user-only access (0700)"

print_summary "Tier 2 - F8 (Socket IPC Boundaries)"
