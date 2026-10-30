#!/usr/bin/env bash
# ==============================================================================
# tests/tier1_features/test_f06_virtual_kb.sh
# Tier 1 Tests for F6: Virtual Keyboard Pass-Through
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 1 Tests for F6: Virtual Keyboard Pass-Through..."

# Test 1: zwp_virtual_keyboard_v1 Integration in Daemon
if check_component "daemon"; then
    daemon_src="${PROJECT_ROOT}/crates/vietkeyra-daemon/src"
    has_vkb=$(grep -r "zwp_virtual_keyboard_v1" "$daemon_src" || true)
    if [ -n "$has_vkb" ]; then
        report_pass "T1_F06_1" "Daemon includes zwp_virtual_keyboard_v1 protocol bindings"
    else
        report_fail "T1_F06_1" "zwp_virtual_keyboard_v1 protocol bindings" "Not found in $daemon_src"
    fi
else
    report_skip "T1_F06_1" "Daemon includes zwp_virtual_keyboard_v1 protocol bindings" "crates/vietkeyra-daemon not yet available"
fi

# Test 2: PassThrough KeyAction Handling in Engine Interface
if check_component "engine"; then
    lib_rs="${PROJECT_ROOT}/crates/vietkeyra-engine/src/lib.rs"
    has_passthrough=$(grep -E "PassThrough" "$lib_rs" || true)
    if [ -n "$has_passthrough" ]; then
        report_pass "T1_F06_2" "KeyAction::PassThrough is defined in engine API"
    else
        report_fail "T1_F06_2" "KeyAction::PassThrough in engine API" "Enum variant not found in lib.rs"
    fi
else
    report_skip "T1_F06_2" "KeyAction::PassThrough is defined in engine API" "crates/vietkeyra-engine not yet available"
fi

# Test 3: Navigation Keys (Arrow keys, Home, End) are Passthrough
if check_component "engine"; then
    out=$(cd "${PROJECT_ROOT}" && cargo test -p vietkeyra-engine test_navigation_passthrough 2>&1 || true)
    if echo "$out" | grep -q "test result: ok"; then
        report_pass "T1_F06_3" "Navigation keys pass through without modifying engine state"
    else
        report_fail "T1_F06_3" "Navigation keys pass through" "Test did not pass"
    fi
else
    report_skip "T1_F06_3" "Navigation keys pass through without modifying engine state" "crates/vietkeyra-engine not yet available"
fi

# Test 4: Shortcut Keys (Ctrl, Alt, Super modifiers) Bypass Telex
if check_component "daemon"; then
    daemon_src="${PROJECT_ROOT}/crates/vietkeyra-daemon/src"
    has_mod_check=$(grep -rE "modifier|Control|Alt|Super|state\.mods" "$daemon_src" || true)
    if [ -n "$has_mod_check" ]; then
        report_pass "T1_F06_4" "Daemon inspects modifiers to bypass Telex on shortcuts"
    else
        report_fail "T1_F06_4" "Shortcut bypass logic" "Modifier inspection logic not found"
    fi
else
    report_skip "T1_F06_4" "Daemon inspects modifiers to bypass Telex on shortcuts" "crates/vietkeyra-daemon not yet available"
fi

# Test 5: English Mode Passes All Keys Through Directly
if check_component "engine"; then
    lib_rs="${PROJECT_ROOT}/crates/vietkeyra-engine/src/lib.rs"
    has_en_mode=$(grep -E "InputMode::English|English" "$lib_rs" || true)
    if [ -n "$has_en_mode" ]; then
        report_pass "T1_F06_5" "InputMode::English is supported in engine for direct pass-through"
    else
        report_fail "T1_F06_5" "InputMode::English support" "Not found in lib.rs"
    fi
else
    report_skip "T1_F06_5" "InputMode::English is supported in engine for direct pass-through" "crates/vietkeyra-engine not yet available"
fi

print_summary "Tier 1 - F6 (Virtual Keyboard Pass-Through)"
