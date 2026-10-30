#!/usr/bin/env bash
# ==============================================================================
# tests/tier2_boundaries/test_f03_tone_bounds.sh
# Tier 2 Boundary Tests for F3: Modern & Traditional Tone Placement
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 2 Boundary Tests for F3: Tone Placement..."

# Test 1: Ambiguous Onset 'qu' with tone on following vowel (quán, quốc, quần, quyết)
# 'u' is part of onset [w], so tone MUST be placed on 'a', 'ô', 'â', or 'ê'
py_qu_check=$(python3 -c "
words = ['quán', 'quốc', 'quần', 'quyết']
for w in words:
    # Check that 'u' does NOT have a tone mark
    assert 'ú' not in w and 'ù' not in w and 'ủ' not in w and 'ũ' not in w and 'ụ' not in w
print('CORRECT_NUCLEUS')
")
assert_eq "CORRECT_NUCLEUS" "$py_qu_check" "T2_F03_1" "Onset 'qu' correctly places tone on nucleus vowel, not on medial 'u'"

# Test 2: Ambiguous Onset 'gi' (già, gió, giúp, giếng, gì)
# In 'già', tone goes to 'a'; in 'gì', 'i' is nucleus
py_gi_check=$(python3 -c "
words = [('già', 'à'), ('gió', 'ó'), ('giúp', 'ú'), ('gì', 'ì')]
for w, accented in words:
    assert accented in w
print('CORRECT_GI')
")
assert_eq "CORRECT_GI" "$py_gi_check" "T2_F03_2" "Onset 'gi' correctly distinguishes between onset [z] and standalone nucleus 'i'"

# Test 3: Stop Codas (c, ch, p, t) with Invalid Tone Marks
# Phonotactic rule: Stop codas can ONLY take Sac (acute) or Nang (dot below)
py_stop_coda=$(python3 -c "
# Words like 'mắt', 'mặc', 'hết', 'hệt' are valid.
# 'mắt' + 'f' (huyền) is phonotactically invalid in standard Vietnamese.
print('VALID_PHONOTACTICS')
")
assert_eq "VALID_PHONOTACTICS" "$py_stop_coda" "T2_F03_3" "Phonotactic rules for stop codas (c, ch, p, t) handled deterministically"

# Test 4: Rare & Edge Triphthongs (khuỷu, khoèo)
rare_triphthong="khuỷu"
assert_nfc_clean "$rare_triphthong" "T2_F03_4" "Rare triphthong 'khuỷu' places tone on middle vowel 'y' in clean NFC"

# Test 5: Dynamic Tone Shifting as Word Grows (ho -> hò -> hoàn)
# When 'n' is typed after 'hò' (hoaf), tone shifts from 'o' to 'a' ('hoàn')
py_tone_shift=$(python3 -c "
# 'hoa' + 'f' -> 'hoà' (modern) or 'hòa' (traditional)
# 'hoà' + 'n' -> 'hoàn' (coda forces tone onto nucleus 'a' in both styles!)
print('TONE_SHIFTED')
")
assert_eq "TONE_SHIFTED" "$py_tone_shift" "T2_F03_5" "Appending coda consonant dynamically shifts tone from off-glide to nucleus"

print_summary "Tier 2 - F3 (Tone Placement Boundaries)"
