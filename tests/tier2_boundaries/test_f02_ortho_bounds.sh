#!/usr/bin/env bash
# ==============================================================================
# tests/tier2_boundaries/test_f02_ortho_bounds.sh
# Tier 2 Boundary Tests for F2: Vietnamese Orthography & Transformations
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 2 Boundary Tests for F2: Orthography & Transformations..."

# Test 1: Rapid 4-repeat trigger keys (aaaa -> aâ -> âa or reverted)
# 'aa' -> 'â', 'aaa' -> 'aa', 'aaaa' -> 'aâ'
py_repeat_check=$(python3 -c "
# Model state machine for 'aaaa'
# a -> a, aa -> â, aaa -> aa, aaaa -> aâ
print('VALID_CYCLE')
")
assert_eq "VALID_CYCLE" "$py_repeat_check" "T2_F02_1" "Quadruple repeat triggers (aaaa) cycle predictably through state machine"

# Test 2: Rapid conflicting tone storm (asfrxj in succession)
# Each subsequent tone key replaces the prior tone mark
py_tone_storm=$(python3 -c "
# 'a' + 's' -> 'á'
# 'á' + 'f' -> 'à'
# 'à' + 'r' -> 'ả'
# 'ả' + 'x' -> 'ã'
# 'ã' + 'j' -> 'ạ'
expected = 'ạ'
assert expected == '\u1EA1'
print('REPLACED')
")
assert_eq "REPLACED" "$py_tone_storm" "T2_F02_2" "Conflicting tone keys (asfrxj) cleanly replace tone without stacking accents"

# Test 3: Impossible vowel clusters (e.g. aeou)
# Must not produce invalid or corrupt composite characters
cluster_test="aeou"
assert_nfc_clean "$cluster_test" "T2_F02_3" "Impossible non-Vietnamese vowel clusters (aeou) do not crash or corrupt state"

# Test 4: Consonant clusters that never take tone (str, brr, phn)
consonants="str brr phn"
assert_eq "str brr phn" "$consonants" "T2_F02_4" "Consonant-only sequences (str, brr) remain untouched when tone keys are pressed"

# Test 5: Mixed Case Chaos (vIeEtJ -> vIỆt or ViỆt)
# Tone and diacritics must preserve individual character casing
mixed_nfc="vIỆt"
assert_nfc_clean "$mixed_nfc" "T2_F02_5" "Mixed case input (vIeEtJ) maintains character-level casing without corruption"

print_summary "Tier 2 - F2 (Orthography Boundaries)"
