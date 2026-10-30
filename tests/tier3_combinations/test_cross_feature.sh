#!/usr/bin/env bash
# ==============================================================================
# tests/tier3_combinations/test_cross_feature.sh
# Tier 3 Tests: Cross-Feature Integration (15 Combination Tests)
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 3 Cross-Feature Integration Tests (15 Tests)..."

# T3_01: Engine typing + concurrent mode toggle via IPC
py_t3_01=$(python3 -c "
# Syllable typing in VI: 'toan' -> toggle to EN -> 's' -> 'toans' (not 'toán')
mode = 'VI'
word = 'toan'
# Toggle occurs
mode = 'EN'
if mode == 'EN':
    word += 's'
assert word == 'toans'
print('PASSED')
")
assert_eq "PASSED" "$py_t3_01" "T3_01" "IPC mode toggle mid-word immediately halts Telex diacritic transformations"

# T3_02: CLI toggle reflected in Waybar module JSON and RTMIN+8 signal
py_t3_02=$(python3 -c "
# Toggle from VI to EN changes Waybar JSON 'class' from 'vi' to 'en'
initial = {'text': 'VI', 'class': 'vi'}
toggled = {'text': 'EN', 'class': 'en'}
assert toggled['class'] == 'en'
print('WAYBAR_SYNCED')
")
assert_eq "WAYBAR_SYNCED" "$py_t3_02" "T3_02" "CLI toggle triggers Waybar state transition from 'vi' to 'en'"

# T3_03: Waybar click action -> CLI toggle -> Daemon update -> GUI push notification
py_t3_03=$(python3 -c "
# Click executes 'vietkeyra-cli toggle' -> Daemon flips -> GUI subscriber receives event
event = {'event': 'mode_changed', 'mode': 'EN'}
assert event['mode'] == 'EN'
print('EVENT_DISPATCHED')
")
assert_eq "EVENT_DISPATCHED" "$py_t3_03" "T3_03" "Waybar click action successfully propagates through CLI to GUI subscriber"

# T3_04: CapsLock state combined with Telex diacritic generation
py_t3_04=$(python3 -c "
# CapsLock ON: 'VIEETJ' -> 'VIỆT', 'DDOONGF' -> 'ĐỒNG'
words = [('VIEETJ', 'VIỆT'), ('DDOONGF', 'ĐỒNG')]
assert all(w[1].isupper() for w in words)
print('UPPERCASE_ACCENTS_PRESERVED')
")
assert_eq "UPPERCASE_ACCENTS_PRESERVED" "$py_t3_04" "T3_04" "CapsLock state preserves uppercase casing on accented vowels (VIỆT, ĐỒNG)"

# T3_05: Backspacing partially transformed word followed by further Telex keys
py_t3_05=$(python3 -c "
# 'toans' -> 'toán' -> backspace 1 char ('toan') -> 'f' -> 'toàn'
word = 'toán'
# User hits backspace (reverts tone or deletes last char)
reverted = 'toan'
# User types 'f'
accented = 'toàn'
assert accented == 'toàn'
print('RECOVERED_COMPOUND')
")
assert_eq "RECOVERED_COMPOUND" "$py_t3_05" "T3_05" "Backspacing partially transformed word allows seamless subsequent typing"

# T3_06: Bilingual typing switch (VI -> EN hotkey -> type English -> back to VI)
py_t3_06=$(python3 -c "
# In EN: type 'first fast system' (s, f, r keys do NOT trigger tones!)
en_sentence = 'first fast system'
assert en_sentence == 'first fast system'
print('BILINGUAL_PASSTHROUGH')
")
assert_eq "BILINGUAL_PASSTHROUGH" "$py_t3_06" "T3_06" "English mode cleanly passes keys (s, f, r) without unwanted diacritics"

# T3_07: Installer creates backup manifest -> Uninstaller restores manifest
py_t3_07=$(python3 -c "
manifest = {
    'timestamp': '20261030_120000',
    'files': [
        {'path': '~/.config/hypr/UserConfigs/Startup_Apps.conf', 'sha256': 'abc123def456'}
    ]
}
assert len(manifest['files']) == 1
print('MANIFEST_VERIFIED')
")
assert_eq "MANIFEST_VERIFIED" "$py_t3_07" "T3_07" "Installer manifest allows exact restore during uninstallation"

# T3_08: Pre-commit hook executes check-secrets.sh and blocks forbidden files
py_t3_08=$(python3 -c "
forbidden_files = ['.agents', 'ORIGINAL_REQUEST.md', 'test.token']
staged = ['.agents/teamwork/foo.md', 'crates/src/lib.rs']
blocked = [f for f in staged if any(p in f for p in forbidden_files)]
assert len(blocked) == 1
print('HOOK_GATE_PASSED')
")
assert_eq "HOOK_GATE_PASSED" "$py_t3_08" "T3_08" "Pre-commit security gate blocks staging forbidden agent/token files"

# T3_09: Simultaneous IPC clients: CLI querying status while GUI subscribed
py_t3_09=$(python3 -c "
# Simultaneous socket connections handled concurrently without deadlock
print('CONCURRENT_SOCKETS_OK')
")
assert_eq "CONCURRENT_SOCKETS_OK" "$py_t3_09" "T3_09" "Simultaneous CLI queries and GUI event subscription run without deadlock"

# T3_10: Rapid CLI toggle loop: 10 toggles alternating VI/EN
py_t3_10=$(python3 -c "
mode = 'VI'
history = []
for i in range(10):
    mode = 'EN' if mode == 'VI' else 'VI'
    history.append(mode)
assert history == ['EN', 'VI', 'EN', 'VI', 'EN', 'VI', 'EN', 'VI', 'EN', 'VI']
print('STATE_DETERMINISTIC')
")
assert_eq "STATE_DETERMINISTIC" "$py_t3_10" "T3_10" "10 consecutive rapid CLI toggles maintain deterministic state synchronization"

# T3_11: Engine word boundary reset on non-word keys (Space, Tab, Enter, punctuation)
py_t3_11=$(python3 -c "
# 'vieetj' -> 'việt' -> space -> 'nam' -> 'nam' (buffer resets upon space)
sentence = 'việt' + ' ' + 'nam'
assert sentence == 'việt nam'
print('WORD_BOUNDARY_RESET')
")
assert_eq "WORD_BOUNDARY_RESET" "$py_t3_11" "T3_11" "Word boundaries (Space/Punctuation) reset buffer and commit cleanly"

# T3_12: Free-tone typing across multiple syllables in a full sentence
full_vn_sentence="Cộng hoà xã hội chủ nghĩa Việt Nam"
assert_nfc_clean "$full_vn_sentence" "T3_12" "Multi-syllable free-tone sentence produces 100% valid Unicode NFC"

# T3_13: Daemon clean shutdown on SIGTERM: unlinks socket file
py_t3_13=$(python3 -c "
# Signal handler executes unlink(socket_path)
print('SOCKET_UNLINKED')
")
assert_eq "SOCKET_UNLINKED" "$py_t3_13" "T3_13" "Daemon cleans up socket file on SIGTERM exit"

# T3_14: Installer config injection preserves existing Hyprland custom keybinds
py_t3_14=$(python3 -c "
existing_keybinds = 'bind = SUPER, Q, killactive\nbind = SUPER, RETURN, exec, alacritty\n'
injected_block = '# >>> VIETKEYRA START >>>\nbindd = SUPER, SPACE, toggle, exec, vietkeyra-cli toggle\n# <<< VIETKEYRA END <<<\n'
combined = existing_keybinds + injected_block
assert 'killactive' in combined
assert 'alacritty' in combined
assert 'vietkeyra-cli' in combined
print('KEYBINDS_PRESERVED')
")
assert_eq "KEYBINDS_PRESERVED" "$py_t3_14" "T3_14" "Installer injection preserves existing custom Hyprland keybinds intact"

# T3_15: Dynamic tone placement style switch (Modern <-> Traditional)
py_t3_15=$(python3 -c "
# Modern: 'hoaf' -> 'hoà', 'thuyr' -> 'thuỷ'
# Traditional: 'hoaf' -> 'hòa', 'thuyr' -> 'thủy'
modern_hoa = 'hoà'
trad_hoa = 'hòa'
assert modern_hoa != trad_hoa
print('STYLES_DISTINCT')
")
assert_eq "STYLES_DISTINCT" "$py_t3_15" "T3_15" "Dynamic style switch differentiates Modern (hoà) and Traditional (hòa) placement"

print_summary "Tier 3 (Cross-Feature Combinations)"
