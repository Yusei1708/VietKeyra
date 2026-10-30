#!/usr/bin/env bash
# ==============================================================================
# tests/common/test_helpers.sh
# Shared assertion utilities, reporters, and environment helpers for VietKeyra
# ==============================================================================

set -uo pipefail

export PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"

# Color codes
if [ -t 1 ]; then
    COLOR_RED="\033[0;31m"
    COLOR_GREEN="\033[0;32m"
    COLOR_YELLOW="\033[1;33m"
    COLOR_BLUE="\033[0;34m"
    COLOR_CYAN="\033[0;36m"
    COLOR_BOLD="\033[1m"
    COLOR_RESET="\033[0m"
else
    COLOR_RED=""
    COLOR_GREEN=""
    COLOR_YELLOW=""
    COLOR_BLUE=""
    COLOR_CYAN=""
    COLOR_BOLD=""
    COLOR_RESET=""
fi

# Test Metrics
TESTS_TOTAL=0
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

declare -a FAILED_TESTS=()
declare -a SKIPPED_TESTS=()

report_pass() {
    local test_id="$1"
    local desc="$2"
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    TESTS_PASSED=$((TESTS_PASSED + 1))
    printf "  ${COLOR_GREEN}✔ PASS${COLOR_RESET} [${test_id}] %s\n" "$desc"
}

report_fail() {
    local test_id="$1"
    local desc="$2"
    local reason="${3:-"Assertion failed"}"
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    TESTS_FAILED=$((TESTS_FAILED + 1))
    FAILED_TESTS+=("${test_id}: ${desc} (${reason})")
    printf "  ${COLOR_RED}✘ FAIL${COLOR_RESET} [${test_id}] %s\n" "$desc"
    printf "         ${COLOR_RED}Reason:${COLOR_RESET} %s\n" "$reason"
}

report_skip() {
    local test_id="$1"
    local desc="$2"
    local reason="${3:-"Dependency not yet available"}"
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    TESTS_SKIPPED=$((TESTS_SKIPPED + 1))
    SKIPPED_TESTS+=("${test_id}: ${desc} (${reason})")
    printf "  ${COLOR_YELLOW}⚠ SKIP${COLOR_RESET} [${test_id}] %s\n" "$desc"
    printf "         ${COLOR_YELLOW}Reason:${COLOR_RESET} %s\n" "$reason"
}

assert_eq() {
    local expected="$1"
    local actual="$2"
    local test_id="$3"
    local desc="$4"

    if [ "$expected" == "$actual" ]; then
        report_pass "$test_id" "$desc"
        return 0
    else
        report_fail "$test_id" "$desc" "Expected '$expected', got '$actual'"
        return 1
    fi
}

assert_not_eq() {
    local unexpected="$1"
    local actual="$2"
    local test_id="$3"
    local desc="$4"

    if [ "$unexpected" != "$actual" ]; then
        report_pass "$test_id" "$desc"
        return 0
    else
        report_fail "$test_id" "$desc" "Value should not equal '$unexpected'"
        return 1
    fi
}

assert_contains() {
    local needle="$1"
    local haystack="$2"
    local test_id="$3"
    local desc="$4"

    if [[ "$haystack" == *"$needle"* ]]; then
        report_pass "$test_id" "$desc"
        return 0
    else
        report_fail "$test_id" "$desc" "Sub-string '$needle' not found in text"
        return 1
    fi
}

assert_not_contains() {
    local needle="$1"
    local haystack="$2"
    local test_id="$3"
    local desc="$4"

    if [[ "$haystack" != *"$needle"* ]]; then
        report_pass "$test_id" "$desc"
        return 0
    else
        report_fail "$test_id" "$desc" "Forbidden string '$needle' found in text"
        return 1
    fi
}

assert_exit_code() {
    local expected="$1"
    local actual="$2"
    local test_id="$3"
    local desc="$4"

    if [ "$expected" -eq "$actual" ]; then
        report_pass "$test_id" "$desc"
        return 0
    else
        report_fail "$test_id" "$desc" "Expected exit code $expected, got $actual"
        return 1
    fi
}

assert_nfc_clean() {
    local text="$1"
    local test_id="$2"
    local desc="$3"

    local nfc_result
    nfc_result=$(python3 -c "
import sys, unicodedata
s = '''$text'''
is_nfc = (unicodedata.normalize('NFC', s) == s)
has_combining = any(0x0300 <= ord(c) <= 0x036F for c in s)
if not is_nfc:
    print('NOT_NFC')
elif has_combining:
    print('HAS_COMBINING')
else:
    print('CLEAN')
" 2>/dev/null || echo "ERROR")

    if [ "$nfc_result" == "CLEAN" ]; then
        report_pass "$test_id" "$desc (NFC Clean)"
        return 0
    elif [ "$nfc_result" == "NOT_NFC" ]; then
        report_fail "$test_id" "$desc" "Output text is not normalized to Unicode NFC"
        return 1
    elif [ "$nfc_result" == "HAS_COMBINING" ]; then
        report_fail "$test_id" "$desc" "Output text contains decomposed combining diacritics (U+0300..=U+036F)"
        return 1
    else
        report_fail "$test_id" "$desc" "Python NFC verification returned $nfc_result"
        return 1
    fi
}

check_component() {
    local component="$1"
    case "$component" in
        "engine")
            [ -f "${PROJECT_ROOT}/crates/vietkeyra-engine/src/lib.rs" ] && grep -q "VietKeyraEngine" "${PROJECT_ROOT}/crates/vietkeyra-engine/src/lib.rs" 2>/dev/null
            ;;
        "daemon")
            [ -f "${PROJECT_ROOT}/crates/vietkeyra-daemon/src/main.rs" ] || [ -f "${PROJECT_ROOT}/target/release/vietkeyra-daemon" ]
            ;;
        "cli")
            [ -f "${PROJECT_ROOT}/crates/vietkeyra-cli/src/main.rs" ] || [ -f "${PROJECT_ROOT}/target/release/vietkeyra-cli" ]
            ;;
        "gui")
            [ -f "${PROJECT_ROOT}/crates/vietkeyra-gui/src/main.rs" ] || [ -f "${PROJECT_ROOT}/target/release/vietkeyra-gui" ]
            ;;
        "waybar")
            [ -f "${PROJECT_ROOT}/configs/waybar/vietkeyra.jsonc" ] || [ -d "${PROJECT_ROOT}/configs/waybar" ]
            ;;
        "installer")
            [ -f "${PROJECT_ROOT}/install.sh" ]
            ;;
        "uninstaller")
            [ -f "${PROJECT_ROOT}/uninstall.sh" ]
            ;;
        "git_security")
            [ -f "${PROJECT_ROOT}/scripts/check-secrets.sh" ] || [ -f "${PROJECT_ROOT}/.gitignore" ]
            ;;
        *)
            return 1
            ;;
    esac
}

print_summary() {
    local suite_name="${1:-"Test Suite"}"
    echo ""
    echo "=============================================================================="
    printf "${COLOR_BOLD}Summary for %s:${COLOR_RESET}\n" "$suite_name"
    printf "  Total Tests:    %d\n" "$TESTS_TOTAL"
    printf "  ${COLOR_GREEN}Passed:         %d${COLOR_RESET}\n" "$TESTS_PASSED"
    printf "  ${COLOR_RED}Failed:         %d${COLOR_RESET}\n" "$TESTS_FAILED"
    printf "  ${COLOR_YELLOW}Skipped:        %d${COLOR_RESET}\n" "$TESTS_SKIPPED"
    
    if [ "$TESTS_TOTAL" -gt 0 ]; then
        local pass_rate=$(( (TESTS_PASSED * 100) / TESTS_TOTAL ))
        printf "  Pass Rate:      %d%%\n" "$pass_rate"
    fi
    echo "=============================================================================="

    if [ "$TESTS_FAILED" -gt 0 ]; then
        echo ""
        printf "${COLOR_RED}Failed Tests Detail:${COLOR_RESET}\n"
        for failure in "${FAILED_TESTS[@]}"; do
            printf "  - %s\n" "$failure"
        done
        return 1
    fi
    return 0
}
