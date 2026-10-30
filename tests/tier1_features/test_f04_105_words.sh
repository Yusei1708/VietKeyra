#!/usr/bin/env bash
# ==============================================================================
# tests/tier1_features/test_f04_105_words.sh
# Tier 1 Tests for F4: 105+ Word Unit Test Suite
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 1 Tests for F4: 105+ Word Unit Test Suite..."

# Test 1: Full 105-word test suite execution via cargo test
if check_component "engine"; then
    out=$(cd "${PROJECT_ROOT}" && cargo test -p vietkeyra-engine test_105_words 2>&1 || true)
    if echo "$out" | grep -q "test result: ok"; then
        report_pass "T1_F04_1" "Full 105-word unit test suite passes"
    else
        report_fail "T1_F04_1" "Full 105-word unit test suite" "Test execution failed"
    fi
else
    report_skip "T1_F04_1" "Full 105-word unit test suite" "crates/vietkeyra-engine not yet available"
fi

# Test 2: Verify test word count >= 105
if check_component "engine"; then
    TEST_FILE="${PROJECT_ROOT}/crates/vietkeyra-engine/src/tests/test_105_words.rs"
    if [ ! -f "$TEST_FILE" ]; then
        TEST_FILE="${PROJECT_ROOT}/crates/vietkeyra-engine/tests/test_105_words.rs"
    fi
    if [ -f "$TEST_FILE" ]; then
        # Count test cases in the test file
        case_count=$(grep -cE '(".*",\s*".*")|test_case!' "$TEST_FILE" || true)
        if [ "$case_count" -ge 105 ]; then
            report_pass "T1_F04_2" "Test suite contains at least 105 distinct test cases (found: $case_count)"
        else
            report_fail "T1_F04_2" "105 distinct test cases" "Found only $case_count test cases"
        fi
    else
        report_fail "T1_F04_2" "105 distinct test cases" "test_105_words.rs file not found"
    fi
else
    report_skip "T1_F04_2" "105 distinct test cases" "crates/vietkeyra-engine not yet available"
fi

# Test 3: Zero Failures in 105-Word Matrix
if check_component "engine"; then
    out=$(cd "${PROJECT_ROOT}" && cargo test -p vietkeyra-engine test_105_words 2>&1 || true)
    if echo "$out" | grep -E "0 failed" | grep -q "ok"; then
        report_pass "T1_F04_3" "0 failures reported in 105-word test run"
    else
        report_fail "T1_F04_3" "0 failures in 105-word test run" "Failed tests detected"
    fi
else
    report_skip "T1_F04_3" "0 failures in 105-word test run" "crates/vietkeyra-engine not yet available"
fi

# Test 4: Uppercase Word Transformation (VIEETJ -> VIỆT, DDOONGF -> ĐỒNG)
if check_component "engine"; then
    out=$(cd "${PROJECT_ROOT}" && cargo test -p vietkeyra-engine test_uppercase 2>&1 || true)
    if echo "$out" | grep -q "test result: ok"; then
        report_pass "T1_F04_4" "Uppercase word transformations pass"
    else
        report_fail "T1_F04_4" "Uppercase word transformations" "Test did not pass"
    fi
else
    report_skip "T1_F04_4" "Uppercase word transformations (VIEETJ -> VIỆT, DDOONGF -> ĐỒNG)" "crates/vietkeyra-engine not yet available"
fi

# Test 5: Special Onset Transformation (già, giếng, gìn)
if check_component "engine"; then
    out=$(cd "${PROJECT_ROOT}" && cargo test -p vietkeyra-engine test_special_onsets 2>&1 || true)
    if echo "$out" | grep -q "test result: ok"; then
        report_pass "T1_F04_5" "Special onset transformations (gi/qu) pass"
    else
        report_fail "T1_F04_5" "Special onset transformations" "Test did not pass"
    fi
else
    report_skip "T1_F04_5" "Special onset transformations (già, giếng, gìn)" "crates/vietkeyra-engine not yet available"
fi

print_summary "Tier 1 - F4 (105+ Word Unit Tests)"
