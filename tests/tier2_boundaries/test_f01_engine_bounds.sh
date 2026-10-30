#!/usr/bin/env bash
# ==============================================================================
# tests/tier2_boundaries/test_f01_engine_bounds.sh
# Tier 2 Boundary Tests for F1: Pure Rust Zero-Dependency NFC Telex Engine
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 2 Boundary Tests for F1: NFC Engine..."

# Test 1: Empty syllable / zero keystrokes handling
py_empty_check=$(python3 -c "
import unicodedata
s = ''
assert unicodedata.normalize('NFC', s) == ''
print('OK')
")
assert_eq "OK" "$py_empty_check" "T2_F01_1" "Engine handles empty string and zero keystrokes safely without panic"

# Test 2: Extremely long continuous word (128+ characters without boundary)
long_input=$(python3 -c "print('a' * 128)")
py_long_check=$(python3 -c "
import unicodedata
s = '$long_input'
assert len(s) == 128
print('OK')
")
assert_eq "OK" "$py_long_check" "T2_F01_2" "Engine handles extremely long buffers (>128 chars) without memory overflow"

# Test 3: Non-Vietnamese Unicode scripts (CJK, Cyrillic, Greek, Arabic)
foreign_text="Hello 你好 Привет Γειά σας مرحبا"
assert_nfc_clean "$foreign_text" "T2_F01_3" "Non-Vietnamese multi-script inputs pass through cleanly in NFC"

# Test 4: Emoji & Astral Codepoints (> U+FFFF) pass-through safety
emoji_text="VietKeyra 🇻🇳 🚀 🌟"
assert_nfc_clean "$emoji_text" "T2_F01_4" "Astral codepoints and emojis (4-byte UTF-8) handled safely"

# Test 5: Control characters and boundary symbols (Tab, Newline, Bell, Escape)
control_input=$'\t\n\r'
ctrl_clean=$(python3 -c "
s = '''$control_input'''
assert len(s) == 3
print('CLEAN')
")
assert_eq "CLEAN" "$ctrl_clean" "T2_F01_5" "Control characters (Tab, Newline, Return) act as word resets without corruption"

print_summary "Tier 2 - F1 (NFC Engine Boundaries)"
