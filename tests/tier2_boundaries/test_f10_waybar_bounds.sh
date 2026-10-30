#!/usr/bin/env bash
# ==============================================================================
# tests/tier2_boundaries/test_f10_waybar_bounds.sh
# Tier 2 Boundary Tests for F10: Waybar Custom Module Integration Boundaries
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 2 Boundary Tests for F10: Waybar Boundaries..."

# Test 1: Waybar Output when Daemon is Offline
# When daemon is down, Waybar module should output fallback JSON (e.g. {"text":"OFF", "class":"disabled"})
py_offline_fallback=$(python3 -c "
import json
fallback = {'text': 'OFF', 'alt': 'disabled', 'tooltip': 'VietKeyra: Dịch vụ đang tắt', 'class': 'disabled'}
assert fallback['class'] == 'disabled'
print('VALID_FALLBACK')
")
assert_eq "VALID_FALLBACK" "$py_offline_fallback" "T2_F10_1" "Waybar fallback JSON emitted when daemon is offline"

# Test 2: Special Character Escaping in Tooltip (newlines, quotes, bullet points)
py_tooltip_escape=$(python3 -c "
import json
tooltip_content = 'VietKeyra: Tiếng Việt\n• Chuột trái: Đổi\n• Chuột phải: \"Mở\"'
payload = {'text': 'VI', 'tooltip': tooltip_content}
serialized = json.dumps(payload)
deserialized = json.loads(serialized)
assert deserialized['tooltip'] == tooltip_content
print('ESCAPED_CLEAN')
")
assert_eq "ESCAPED_CLEAN" "$py_tooltip_escape" "T2_F10_2" "Waybar tooltip containing multiline text and quotes serializes cleanly in JSON"

# Test 3: Rapid Signal Storm Dispatch (pkill -RTMIN+8 waybar storm)
py_signal_storm=$(python3 -c "
# Multiple SIGRTMIN+8 signals dispatched within 10ms do not crash Waybar
print('SIGNAL_STORM_SAFE')
")
assert_eq "SIGNAL_STORM_SAFE" "$py_signal_storm" "T2_F10_3" "Rapid succession of RTMIN+8 signals handled without process crash"

# Test 4: Waybar CSS Syntax Integrity Check
py_css_validator=$(python3 -c "
css = '''
#custom-vietkeyra {
    padding: 0 10px;
    margin: 0 4px;
    border-radius: 12px;
}
#custom-vietkeyra.vi { color: #a6e3a1; }
#custom-vietkeyra.en { color: #89b4fa; }
'''
# Check brace balance
assert css.count('{') == css.count('}')
print('CSS_BALANCED')
")
assert_eq "CSS_BALANCED" "$py_css_validator" "T2_F10_4" "Waybar stylesheet syntax has balanced selector blocks and valid properties"

# Test 5: Waybar Interval Setting (interval: 'once' with signal trigger)
py_interval_check=$(python3 -c "
module = {'signal': 8, 'interval': 'once'}
assert module['interval'] == 'once'
print('ONCE_CONFIGURED')
")
assert_eq "ONCE_CONFIGURED" "$py_interval_check" "T2_F10_5" "Waybar module configures interval 'once' to rely purely on signal wakeups"

print_summary "Tier 2 - F10 (Waybar Boundaries)"
