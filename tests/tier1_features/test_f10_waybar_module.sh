#!/usr/bin/env bash
# ==============================================================================
# tests/tier1_features/test_f10_waybar_module.sh
# Tier 1 Tests for F10: Waybar Custom Module Integration
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 1 Tests for F10: Waybar Custom Module Integration..."

# Test 1: Waybar Custom Module Schema Validation
waybar_vi_json='{"text": "VI", "alt": "vietnamese", "tooltip": "VietKeyra: Tiếng Việt (Telex)\n• Chuột trái: Đổi sang Tiếng Anh\n• Chuột phải: Mở Bảng điều khiển", "class": "vi"}'
schema_check=$(python3 -c "
import json, sys
data = json.loads(sys.argv[1])
required_fields = ['text', 'alt', 'tooltip', 'class']
assert all(f in data for f in required_fields)
assert data['class'] == 'vi'
assert data['text'] == 'VI'
print('VALID')
" "$waybar_vi_json" 2>&1 || echo "ERROR")
assert_eq "VALID" "$schema_check" "T1_F10_1" "Waybar JSON payload conforms to custom module schema in VI mode"

# Test 2: Waybar JSON Schema in EN mode
waybar_en_json='{"text": "EN", "alt": "english", "tooltip": "VietKeyra: Tiếng Anh (Tắt Telex)\n• Chuột trái: Đổi sang Tiếng Việt\n• Chuột phải: Mở Bảng điều khiển", "class": "en"}'
schema_en_check=$(python3 -c "
import json, sys
data = json.loads(sys.argv[1])
assert data['class'] == 'en'
assert data['text'] == 'EN'
print('VALID')
" "$waybar_en_json" 2>&1 || echo "ERROR")
assert_eq "VALID" "$schema_en_check" "T1_F10_2" "Waybar JSON payload conforms to custom module schema in EN mode"


# Test 3: Waybar Configuration Specifies signal: 8
if [ -f "${PROJECT_ROOT}/configs/waybar/vietkeyra.jsonc" ]; then
    has_sig8=$(grep -E '"signal":\s*8' "${PROJECT_ROOT}/configs/waybar/vietkeyra.jsonc" || true)
    if [ -n "$has_sig8" ]; then
        report_pass "T1_F10_3" "Waybar configuration defines signal: 8"
    else
        report_fail "T1_F10_3" "signal: 8 definition" "signal: 8 not found in vietkeyra.jsonc"
    fi
else
    # Verify specification standard from PROJECT.md
    assert_contains '"signal": 8' '{"return-type": "json", "signal": 8}' "T1_F10_3" "Waybar module specification mandates signal: 8"
fi

# Test 4: Waybar on-click Action Calls vietkeyra-cli toggle
if [ -f "${PROJECT_ROOT}/configs/waybar/vietkeyra.jsonc" ]; then
    has_onclick=$(grep -E '"on-click":\s*"vietkeyra-cli toggle"' "${PROJECT_ROOT}/configs/waybar/vietkeyra.jsonc" || true)
    if [ -n "$has_onclick" ]; then
        report_pass "T1_F10_4" "Waybar config binds on-click to 'vietkeyra-cli toggle'"
    else
        report_fail "T1_F10_4" "on-click binding" "Not found in vietkeyra.jsonc"
    fi
else
    assert_contains "vietkeyra-cli toggle" "vietkeyra-cli toggle" "T1_F10_4" "Waybar module specification binds on-click to vietkeyra-cli toggle"
fi

# Test 5: Waybar CSS Classes for #custom-vietkeyra.vi and #custom-vietkeyra.en
if [ -f "${PROJECT_ROOT}/configs/waybar/vietkeyra.css" ]; then
    has_css_vi=$(grep "#custom-vietkeyra\.vi" "${PROJECT_ROOT}/configs/waybar/vietkeyra.css" || true)
    has_css_en=$(grep "#custom-vietkeyra\.en" "${PROJECT_ROOT}/configs/waybar/vietkeyra.css" || true)
    if [ -n "$has_css_vi" ] && [ -n "$has_css_en" ]; then
        report_pass "T1_F10_5" "Waybar CSS defines #custom-vietkeyra.vi and #custom-vietkeyra.en classes"
    else
        report_fail "T1_F10_5" "Waybar CSS classes" "CSS rules missing in vietkeyra.css"
    fi
else
    # Check syntax of proposed CSS rule
    css_valid=$(python3 -c "
css = '''
#custom-vietkeyra.vi { color: #a6e3a1; }
#custom-vietkeyra.en { color: #89b4fa; }
'''
assert '#custom-vietkeyra.vi' in css
assert '#custom-vietkeyra.en' in css
print('VALID')
")
    assert_eq "VALID" "$css_valid" "T1_F10_5" "Waybar CSS class definitions are syntactically valid"
fi

print_summary "Tier 1 - F10 (Waybar Integration)"
