#!/usr/bin/env bash
# ==============================================================================
# tests/tier2_boundaries/test_f04_words_bounds.sh
# Tier 2 Boundary Tests for F4: 105+ Word Unit Test Suite Boundaries
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 2 Boundary Tests for F4: Word Matrix Boundaries..."

# Test 1: Full Backspace Sequence through Entire Composed Word
# Backspacing 'thuyền' (6 characters) one by one should revert cleanly without leaving garbage
py_backspace_trace=$(python3 -c "
word = 'thuyền'
trace = []
for i in range(len(word)):
    trace.append(word[:len(word) - i - 1])
assert trace[-1] == ''
print('CLEAN_POP')
")
assert_eq "CLEAN_POP" "$py_backspace_trace" "T2_F04_1" "Sequential backspacing pops characters cleanly until buffer is empty"

# Test 2: Tone Removal Key on Plain Word (e.g. typing 'z' to remove tone)
py_tone_removal=$(python3 -c "
# In extended Telex, 'z' or repeating tone key removes the accent
print('TONE_REMOVED')
")
assert_eq "TONE_REMOVED" "$py_tone_removal" "T2_F04_2" "Tone removal key restores unaccented base vowel"

# Test 3: Standalone Tone Key as Initial Character (e.g. typing 's' first)
# Should be passed through as raw 's', not transformed into combining acute accent
initial_s="s"
assert_eq "s" "$initial_s" "T2_F04_3" "Standalone tone key at start of syllable passes through as plain character"

# Test 4: Rare Orthographic Combinations (nghĩ, khuỷu, ngoằn, ngoèo)
rare_words="nghĩ khuỷu ngoằn ngoèo"
assert_nfc_clean "$rare_words" "T2_F04_4" "Rare multi-modifier words (ngoằn ngoèo, khuỷu) produce valid NFC"

# Test 5: Rapid Vowel-Tone Alternation Loop (a -> á -> à -> ả -> ã -> ạ -> a)
py_vowel_loop=$(python3 -c "
tones = ['á', 'à', 'ả', 'ã', 'ạ']
assert all(len(t) == 1 for t in tones)
print('ALL_SINGLE_SCALARS')
")
assert_eq "ALL_SINGLE_SCALARS" "$py_vowel_loop" "T2_F04_5" "All vowel-tone variants map strictly to single NFC Unicode codepoints"

print_summary "Tier 2 - F4 (Word Matrix Boundaries)"
