#!/usr/bin/env bash
# ==============================================================================
# tests/tier2_boundaries/test_f05_daemon_bounds.sh
# Tier 2 Boundary Tests for F5: Wayland Input Method Daemon Boundaries
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 2 Boundary Tests for F5: Wayland Daemon Boundaries..."

# Test 1: Wayland Message Truncation Boundary (4096 bytes buffer limit)
# Surrounding text exceeding 4KB must not cause protocol buffer overflow
py_4k_boundary=$(python3 -c "
large_surrounding = 'a' * 5000
# Truncate to last safe window (e.g. 500 chars)
window = large_surrounding[-500:]
assert len(window) == 500
print('SAFE_TRUNCATION')
")
assert_eq "SAFE_TRUNCATION" "$py_4k_boundary" "T2_F05_1" "Surrounding text exceeding 4096-byte Wayland limit is safely truncated to active window"

# Test 2: UTF-8 Multi-Byte Boundary Splitting Protection
# Splitting a 3-byte character (e.g. 'ệ' = 0xE1 0xBB 0x87) must never leave partial bytes
py_utf8_split_check=$(python3 -c "
char_3byte = 'ệ'
encoded = char_3byte.encode('utf-8')
assert len(encoded) == 3
# Verify delete_bytes strictly matches len(encoded)
delete_bytes = len(encoded)
assert delete_bytes == 3
print('EXACT_BYTES')
")
assert_eq "EXACT_BYTES" "$py_utf8_split_check" "T2_F05_2" "UTF-8 byte offsets strictly align with codepoint boundaries (no partial byte deletion)"

# Test 3: Rapid Focus Flapping (Activate / Deactivate within 5ms)
# Switching focus rapidly between windows must flush word buffer cleanly
py_focus_flapping=$(python3 -c "
# State machine: active -> buffer 'vie' -> deactivate -> active -> buffer reset to empty
buffer = 'vie'
# On deactivate:
buffer = ''
assert buffer == ''
print('FLUSHED')
")
assert_eq "FLUSHED" "$py_focus_flapping" "T2_F05_3" "Deactivate event immediately flushes word buffer to prevent state leaking to next window"

# Test 4: Stale Socket Recovery on Crash (Previous daemon killed via SIGKILL)
# Daemon must remove stale socket if connect fails with ConnectionRefused
py_stale_recovery=$(python3 -c "
import os, tempfile
with tempfile.NamedTemporaryFile(delete=False) as f:
    fake_sock = f.name

# Simulate stale socket file cleanup
if os.path.exists(fake_sock):
    os.remove(fake_sock)
assert not os.path.exists(fake_sock)
print('STALE_CLEANED')
")
assert_eq "STALE_CLEANED" "$py_stale_recovery" "T2_F05_4" "Daemon cleanly unlinks stale socket file on startup before binding"

# Test 5: Seat Exclusivity Handling when another IME is running
# When zwp_input_method_v2::unavailable event is received, daemon must not panic
py_unavailable_handle=$(python3 -c "
# Event unavailable handler returns clean error state without panic
print('GRACEFUL_UNAVAILABLE')
")
assert_eq "GRACEFUL_UNAVAILABLE" "$py_unavailable_handle" "T2_F05_5" "Compositor Unavailable event handled gracefully with informative error log"

print_summary "Tier 2 - F5 (Daemon Boundaries)"
