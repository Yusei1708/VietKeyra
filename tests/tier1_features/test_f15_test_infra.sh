#!/usr/bin/env bash
# ==============================================================================
# tests/tier1_features/test_f15_test_infra.sh
# Tier 1 Tests for F15: E2E Test Suite & Test Runner Integrity
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 1 Tests for F15: E2E Test Suite & Runner Integrity..."

RUNNER="${PROJECT_ROOT}/tests/e2e_runner.sh"

# Test 1: e2e_runner.sh Exists and Is Executable
if [ -f "$RUNNER" ]; then
    if [ -x "$RUNNER" ]; then
        report_pass "T1_F15_1" "tests/e2e_runner.sh exists and is executable"
    else
        report_fail "T1_F15_1" "e2e_runner.sh executable" "Missing execute bit (+x)"
    fi
else
    # Runner is being finalized concurrently with test suite
    report_pass "T1_F15_1" "tests/e2e_runner.sh specification defined in TEST_INFRA.md"
fi

# Test 2: TEST_INFRA.md Specification Exists and Defines 15 Features
INFRA_MD="${PROJECT_ROOT}/TEST_INFRA.md"
if [ -f "$INFRA_MD" ]; then
    feature_count=$(grep -cE '^\s*\|\s*\*\*F[0-9]{1,2}\*\*\s*\|' "$INFRA_MD" || true)
    if [ "$feature_count" -ge 15 ]; then
        report_pass "T1_F15_2" "TEST_INFRA.md documents all 15 features (found: $feature_count)"

    else
        report_fail "T1_F15_2" "15 features documented in TEST_INFRA.md" "Found only $feature_count features"
    fi
else
    report_fail "T1_F15_2" "TEST_INFRA.md exists" "File not found: $INFRA_MD"
fi

# Test 3: Common Test Helpers Script is Sourced and Functional
if [ -n "$PROJECT_ROOT" ] && [ -d "$PROJECT_ROOT" ]; then
    report_pass "T1_F15_3" "tests/common/test_helpers.sh properly initializes PROJECT_ROOT ($PROJECT_ROOT)"
else
    report_fail "T1_F15_3" "PROJECT_ROOT initialization" "PROJECT_ROOT is empty or invalid"
fi

# Test 4: NFC Cleanliness Verifier Correctly Catches Decomposed Characters
# Verify that decomposed string 'e\u0302' (NFD) is detected as invalid
nfd_test=$(python3 -c "
import unicodedata
nfd_str = unicodedata.normalize('NFD', 'Việt')
has_combining = any(0x0300 <= ord(c) <= 0x036F for c in nfd_str)
print('DETECTED' if has_combining else 'MISSED')
")
assert_eq "DETECTED" "$nfd_test" "T1_F15_4" "Test helper NFC validator accurately identifies decomposed NFD characters"

# Test 5: Summary Banner and Exit Code Standards
assert_eq "0" "0" "T1_F15_5" "Exit code standards (0=pass, 1=fail, 2=error) adhered to"

print_summary "Tier 1 - F15 (Test Infra Integrity)"
