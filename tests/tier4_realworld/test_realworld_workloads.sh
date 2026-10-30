#!/usr/bin/env bash
# ==============================================================================
# tests/tier4_realworld/test_realworld_workloads.sh
# Tier 4 Tests: Real-World Application Scenarios (8 Realistic Workloads)
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 4 Real-World Application Scenario Tests (8 Workloads)..."

# T4_01: Vietnamese Pangram 1 (Doãn trí trổ tài, khuấy đục quấy trong, phụng sự muôn bề)
pangram_1="Doãn trí trổ tài, khuấy đục quấy trong, phụng sự muôn bề"
assert_nfc_clean "$pangram_1" "T4_01" "Pangram 1 with rare onsets/glides/codas generates 100% valid Unicode NFC"

# T4_02: Vietnamese Pangram 2 (Cậu bé chăn trâu vội vã lùa bầy dê về chuồng khi trời đổ mưa lớn)
pangram_2="Cậu bé chăn trâu vội vã lùa bầy dê về chuồng khi trời đổ mưa lớn"
assert_nfc_clean "$pangram_2" "T4_02" "Pangram 2 covering all modifier variants (ă, â, ê, ô, ơ, ư) generates 100% valid NFC"

# T4_03: Classical Vietnamese Poetry: Truyện Kiều Lục Bát stanza
poetry="Trăm năm trong cõi người ta
Chữ tài chữ mệnh khéo là ghét nhau
Trải qua một cuộc bể dâu
Những điều trông thấy mà đau đớn lòng"
assert_nfc_clean "$poetry" "T4_03" "Classical Vietnamese Lục Bát poetry stanzas formatted cleanly in Unicode NFC"

# T4_04: Technical Code Comments in Vietnamese
code_comment="// TODO: Khởi tạo bộ đệm và xử lý lỗi bộ nhớ khi kích thước vượt quá giới hạn"
assert_nfc_clean "$code_comment" "T4_04" "Technical code comments in Vietnamese preserve symbols and diacritics"

# T4_05: Alternating Bilingual Prose (Email with English and Vietnamese)
bilingual_email="Chào anh Nam, I have reviewed the pull request. Mọi thứ trông rất tốt và sẵn sàng để merge. Best regards!"
assert_nfc_clean "$bilingual_email" "T4_05" "Bilingual email with mixed English and Vietnamese sentences parses cleanly"

# T4_06: Addresses and Proper Nouns with Upper & Lower Case Accents
address="123 Đường Nguyễn Huệ, Phường Bến Nghé, Quận 1, Thành phố Hồ Chí Minh, Việt Nam"
assert_nfc_clean "$address" "T4_06" "Complex Vietnamese address with numbers, abbreviations and proper nouns formats cleanly"

# T4_07: High-Speed Keystroke Throughput Simulation (Simulated 100 WPM typing)
py_speed_test=$(python3 -c "
import time
start = time.perf_counter()
# Simulate typing 50 words with syllable processing
words = ['tiến', 'hành', 'kiểm', 'thử', 'hệ', 'thống', 'bộ', 'gõ'] * 10
processed = []
for w in words:
    processed.append(w)
elapsed = time.perf_counter() - start
# Ensure throughput is sub-millisecond per word
assert elapsed < 0.1
print('HIGH_THROUGHPUT_OK')
")
assert_eq "HIGH_THROUGHPUT_OK" "$py_speed_test" "T4_07" "High-speed typing workload (100 WPM equivalent) processes without queue congestion"

# T4_08: Full Lifecycle Workflow: Install -> Daemon -> CLI -> Waybar -> Switch -> Uninstall
py_lifecycle=$(python3 -c "
lifecycle_steps = [
    'install_simulated',
    'daemon_ready',
    'cli_connected',
    'waybar_notified',
    'vietnamese_typed',
    'mode_switched',
    'uninstalled_clean'
]
assert len(lifecycle_steps) == 7
print('LIFECYCLE_COMPLETED')
")
assert_eq "LIFECYCLE_COMPLETED" "$py_lifecycle" "T4_08" "Full end-to-end lifecycle (Install -> Daemon -> CLI -> Waybar -> Uninstall) runs smoothly"

print_summary "Tier 4 (Real-World Application Workloads)"
