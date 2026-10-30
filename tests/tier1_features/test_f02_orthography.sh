#!/usr/bin/env bash
# ==============================================================================
# tests/tier1_features/test_f02_orthography.sh
# Tier 1 Tests for F2: Vietnamese Orthography & Transformations
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 1 Tests for F2: Vietnamese Orthography & Transformations..."

# Test 1: Circumflex Transformations (aa->â, ee->ê, oo->ô)
if check_component "engine"; then
    out=$(cd "${PROJECT_ROOT}" && cargo test -p vietkeyra-engine test_circumflex 2>&1 || true)
    if echo "$out" | grep -q "test result: ok"; then
        report_pass "T1_F02_1" "Circumflex transforms (aa->â, ee->ê, oo->ô) pass"
    else
        report_fail "T1_F02_1" "Circumflex transforms" "Test did not pass"
    fi
else
    report_skip "T1_F02_1" "Circumflex transforms (aa->â, ee->ê, oo->ô)" "crates/vietkeyra-engine not yet available"
fi

# Test 2: Breve & Horn Transformations (aw->ă, ow->ơ, uw->ư, w->ư)
if check_component "engine"; then
    out=$(cd "${PROJECT_ROOT}" && cargo test -p vietkeyra-engine test_breve_horn 2>&1 || true)
    if echo "$out" | grep -q "test result: ok"; then
        report_pass "T1_F02_2" "Breve and horn transforms (aw->ă, ow->ơ, uw->ư, w->ư) pass"
    else
        report_fail "T1_F02_2" "Breve and horn transforms" "Test did not pass"
    fi
else
    report_skip "T1_F02_2" "Breve and horn transforms (aw->ă, ow->ơ, uw->ư, w->ư)" "crates/vietkeyra-engine not yet available"
fi

# Test 3: D-stroke Transformation (dd->đ, DD->Đ)
if check_component "engine"; then
    out=$(cd "${PROJECT_ROOT}" && cargo test -p vietkeyra-engine test_d_stroke 2>&1 || true)
    if echo "$out" | grep -q "test result: ok"; then
        report_pass "T1_F02_3" "D-stroke transform (dd->đ, DD->Đ) pass"
    else
        report_fail "T1_F02_3" "D-stroke transform" "Test did not pass"
    fi
else
    report_skip "T1_F02_3" "D-stroke transform (dd->đ, DD->Đ)" "crates/vietkeyra-engine not yet available"
fi

# Test 4: Five Tone Marks (s, f, r, x, j)
if check_component "engine"; then
    out=$(cd "${PROJECT_ROOT}" && cargo test -p vietkeyra-engine test_tones 2>&1 || true)
    if echo "$out" | grep -q "test result: ok"; then
        report_pass "T1_F02_4" "Five tone marks (s, f, r, x, j) pass"
    else
        report_fail "T1_F02_4" "Five tone marks" "Test did not pass"
    fi
else
    report_skip "T1_F02_4" "Five tone marks (s, f, r, x, j)" "crates/vietkeyra-engine not yet available"
fi

# Test 5: Repeat Key Undo / Reversion (as + s -> as, aa + a -> aa, dd + d -> dd)
if check_component "engine"; then
    out=$(cd "${PROJECT_ROOT}" && cargo test -p vietkeyra-engine test_undo 2>&1 || true)
    if echo "$out" | grep -q "test result: ok"; then
        report_pass "T1_F02_5" "Repeat-key undo state machine passes"
    else
        report_fail "T1_F02_5" "Repeat-key undo state machine" "Test did not pass"
    fi
else
    report_skip "T1_F02_5" "Repeat-key undo state machine (as+s->as, aa+a->aa, dd+d->dd)" "crates/vietkeyra-engine not yet available"
fi

print_summary "Tier 1 - F2 (Vietnamese Orthography)"
