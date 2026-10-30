#!/usr/bin/env bash
# ==============================================================================
# tests/tier1_features/test_f03_tone_placement.sh
# Tier 1 Tests for F3: Modern & Traditional Tone Placement
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 1 Tests for F3: Modern & Traditional Tone Placement..."

# Test 1: Modern Standard Tone Placement on oa, oe, uy (hoà, khoè, thuỷ)
if check_component "engine"; then
    out=$(cd "${PROJECT_ROOT}" && cargo test -p vietkeyra-engine test_modern_tone 2>&1 || true)
    if echo "$out" | grep -q "test result: ok"; then
        report_pass "T1_F03_1" "Modern tone placement (hoà, khoè, thuỷ) passes"
    else
        report_fail "T1_F03_1" "Modern tone placement" "Test did not pass"
    fi
else
    report_skip "T1_F03_1" "Modern tone placement (hoà, khoè, thuỷ)" "crates/vietkeyra-engine not yet available"
fi

# Test 2: Traditional Tone Placement (hòa, khỏe, thủy)
if check_component "engine"; then
    out=$(cd "${PROJECT_ROOT}" && cargo test -p vietkeyra-engine test_traditional_tone 2>&1 || true)
    if echo "$out" | grep -q "test result: ok"; then
        report_pass "T1_F03_2" "Traditional tone placement (hòa, khỏe, thủy) passes"
    else
        report_fail "T1_F03_2" "Traditional tone placement" "Test did not pass"
    fi
else
    report_skip "T1_F03_2" "Traditional tone placement (hòa, khỏe, thủy)" "crates/vietkeyra-engine not yet available"
fi

# Test 3: Closed Syllables with Coda (toán, hoàn, tiến, cuốn)
if check_component "engine"; then
    out=$(cd "${PROJECT_ROOT}" && cargo test -p vietkeyra-engine test_closed_syllables 2>&1 || true)
    if echo "$out" | grep -q "test result: ok"; then
        report_pass "T1_F03_3" "Closed syllables tone placement passes"
    else
        report_fail "T1_F03_3" "Closed syllables tone placement" "Test did not pass"
    fi
else
    report_skip "T1_F03_3" "Closed syllables tone placement (toán, hoàn, tiến, cuốn)" "crates/vietkeyra-engine not yet available"
fi

# Test 4: Open Diphthongs (mía, của, lụa)
if check_component "engine"; then
    out=$(cd "${PROJECT_ROOT}" && cargo test -p vietkeyra-engine test_open_diphthongs 2>&1 || true)
    if echo "$out" | grep -q "test result: ok"; then
        report_pass "T1_F03_4" "Open diphthongs (mía, của, lụa) tone placement passes"
    else
        report_fail "T1_F03_4" "Open diphthongs tone placement" "Test did not pass"
    fi
else
    report_skip "T1_F03_4" "Open diphthongs (mía, của, lụa)" "crates/vietkeyra-engine not yet available"
fi

# Test 5: Triphthongs Middle Vowel Placement (ngoài, xoáy, chuỗi, rượu, nhiều)
if check_component "engine"; then
    out=$(cd "${PROJECT_ROOT}" && cargo test -p vietkeyra-engine test_triphthongs 2>&1 || true)
    if echo "$out" | grep -q "test result: ok"; then
        report_pass "T1_F03_5" "Triphthong middle vowel tone placement passes"
    else
        report_fail "T1_F03_5" "Triphthong middle vowel tone placement" "Test did not pass"
    fi
else
    report_skip "T1_F03_5" "Triphthong middle vowel tone placement (ngoài, xoáy, chuỗi, rượu)" "crates/vietkeyra-engine not yet available"
fi

print_summary "Tier 1 - F3 (Tone Placement)"
