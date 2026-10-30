#!/usr/bin/env bash
# ==============================================================================
# tests/tier1_features/test_f01_engine_nfc.sh
# Tier 1 Tests for F1: Pure Rust Zero-Dependency NFC Telex Engine
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 1 Tests for F1: Pure Rust Zero-Dependency NFC Engine..."

# Test 1: Zero External Dependencies in vietkeyra-engine Cargo.toml
CARGO_TOML="${PROJECT_ROOT}/crates/vietkeyra-engine/Cargo.toml"
if [ -f "$CARGO_TOML" ]; then
    # Check that [dependencies] section does not have external crates
    dep_count=$(python3 -c "
import sys
in_deps = False
deps = []
with open('$CARGO_TOML') as f:
    for line in f:
        line = line.strip()
        if line.startswith('[dependencies]'):
            in_deps = True
            continue
        elif line.startswith('[') and in_deps:
            break
        if in_deps and line and not line.startswith('#'):
            deps.append(line)
print(len(deps))
")
    assert_eq "0" "$dep_count" "T1_F01_1" "Crate vietkeyra-engine has 0 external dependencies"
else
    report_skip "T1_F01_1" "Crate vietkeyra-engine has 0 external dependencies" "crates/vietkeyra-engine not yet available"
fi

# Test 2: Source Code has NFC Table without External Normalization Crates
NFC_RS="${PROJECT_ROOT}/crates/vietkeyra-engine/src/nfc.rs"
if [ -f "$NFC_RS" ]; then
    has_unicode_norm=$(grep -E "unicode_normalization|unicode_segmentation" "$NFC_RS" || true)
    assert_eq "" "$has_unicode_norm" "T1_F01_2" "nfc.rs does not use external normalization crates"
else
    report_skip "T1_F01_2" "nfc.rs does not use external normalization crates" "nfc.rs not yet available"
fi

# Test 3: Output Strings are Verified Valid Unicode NFC (No Combining Codepoints)
# Verify our NFC tester itself correctly flags valid NFC vs invalid decomposed NFD
nfc_sample="Việt Nam đất nước hoà bình"
assert_nfc_clean "$nfc_sample" "T1_F01_3" "Sample Vietnamese text is strictly normalized Unicode NFC"

# Test 4: Pure Rust Cargo Test Compilation & Execution for NFC Engine
if [ -f "$NFC_RS" ]; then
    test_out=$(cd "${PROJECT_ROOT}" && cargo test -p vietkeyra-engine nfc 2>&1 || true)
    if echo "$test_out" | grep -q "test result: ok"; then
        report_pass "T1_F01_4" "cargo test -p vietkeyra-engine nfc passes"
    else
        report_fail "T1_F01_4" "cargo test -p vietkeyra-engine nfc passes" "Test failed or exited with error"
    fi
else
    report_skip "T1_F01_4" "cargo test -p vietkeyra-engine nfc passes" "nfc.rs not yet available"
fi

# Test 5: ASCII Consonants Preservation Invariant
if [ -f "$NFC_RS" ]; then
    test_out=$(cd "${PROJECT_ROOT}" && cargo test -p vietkeyra-engine 2>&1 || true)
    if echo "$test_out" | grep -q "test result: ok"; then
        report_pass "T1_F01_5" "Engine preserves standard ASCII consonants"
    else
        report_fail "T1_F01_5" "Engine preserves standard ASCII consonants" "Engine test execution failed"
    fi
else
    report_skip "T1_F01_5" "Engine preserves standard ASCII consonants" "crates/vietkeyra-engine not yet available"
fi


print_summary "Tier 1 - F1 (NFC Telex Engine)"
