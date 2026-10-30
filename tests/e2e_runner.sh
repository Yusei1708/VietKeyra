#!/usr/bin/env bash
# ==============================================================================
# tests/e2e_runner.sh
# Comprehensive 4-Tier Automated End-to-End Test Runner for VietKeyra
# ==============================================================================

set -uo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PROJECT_ROOT

# CLI Arguments
RUN_ALL=true
TARGET_TIER=""
TARGET_FEATURE=""
OUTPUT_JSON=false
VERBOSE=false

show_help() {
    cat <<EOF
VietKeyra E2E Test Suite Runner

Usage:
  ./tests/e2e_runner.sh [OPTIONS]

Options:
  --all               Run all test tiers (Tiers 1-4) [default]
  --tier <1|2|3|4>    Execute only the specified test tier
  --feature <F1..F15> Run tests covering a specific feature ID
  --json              Output aggregated results in JSON format
  --verbose           Show detailed command outputs
  --help              Display this help message and exit

Examples:
  ./tests/e2e_runner.sh
  ./tests/e2e_runner.sh --tier 1
  ./tests/e2e_runner.sh --feature F01
  ./tests/e2e_runner.sh --json
EOF
    exit 0
}

# Parse CLI arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --help|-h)
            show_help
            ;;
        --all)
            RUN_ALL=true
            shift
            ;;
        --tier)
            TARGET_TIER="$2"
            RUN_ALL=false
            shift 2
            ;;
        --feature)
            TARGET_FEATURE="$2"
            RUN_ALL=false
            shift 2
            ;;
        --json)
            OUTPUT_JSON=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        *)
            echo "Unknown option: $1" >&2
            show_help
            ;;
    esac
done

# Source test helpers
source "${PROJECT_ROOT}/tests/common/test_helpers.sh"

echo "=============================================================================="
printf "${COLOR_BOLD}${COLOR_CYAN}VietKeyra End-to-End Test Suite Runner${COLOR_RESET}\n"
printf "Project Root: %s\n" "$PROJECT_ROOT"
printf "Timestamp:    %s\n" "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo "=============================================================================="

# Progressive Capability Detection
echo "Inspecting repository component readiness..."
COMPONENTS_READY=()
COMPONENTS_PENDING=()

for comp in engine daemon cli gui waybar installer uninstaller git_security; do
    if check_component "$comp"; then
        COMPONENTS_READY+=("$comp")
        printf "  [✔] %-15s : Available\n" "$comp"
    else
        COMPONENTS_PENDING+=("$comp")
        printf "  [⏳] %-15s : Pending / In Progress\n" "$comp"
    fi
done
echo "------------------------------------------------------------------------------"

OVERALL_TOTAL=0
OVERALL_PASSED=0
OVERALL_FAILED=0
OVERALL_SKIPPED=0

execute_script() {
    local script_path="$1"
    local script_name
    script_name=$(basename "$script_path")

    if [ ! -f "$script_path" ]; then
        echo "Error: Test script not found: $script_path" >&2
        return 1
    fi

    # Execute script capturing stdout/stderr
    local script_output
    local exit_code=0
    script_output=$(bash "$script_path" 2>&1) || exit_code=$?

    if [ "$VERBOSE" = true ] || [ "$exit_code" -ne 0 ]; then
        echo "$script_output"
    else
        # Extract individual PASS/FAIL/SKIP lines
        echo "$script_output" | grep -E "^\s*([✔✘⚠]|\s*Summary)" || true
    fi

    # Parse counts from summary
    local t_total t_pass t_fail t_skip
    t_total=$(echo "$script_output" | grep -E "^\s*Total Tests:" | awk '{print $3}' || echo "0")
    t_pass=$(echo "$script_output" | grep -E "^\s*Passed:" | awk '{print $2}' || echo "0")
    t_fail=$(echo "$script_output" | grep -E "^\s*Failed:" | awk '{print $2}' || echo "0")
    t_skip=$(echo "$script_output" | grep -E "^\s*Skipped:" | awk '{print $2}' || echo "0")

    t_total=${t_total:-0}
    t_pass=${t_pass:-0}
    t_fail=${t_fail:-0}
    t_skip=${t_skip:-0}

    OVERALL_TOTAL=$((OVERALL_TOTAL + t_total))
    OVERALL_PASSED=$((OVERALL_PASSED + t_pass))
    OVERALL_FAILED=$((OVERALL_FAILED + t_fail))
    OVERALL_SKIPPED=$((OVERALL_SKIPPED + t_skip))

    return $exit_code
}

# Run Tier 1
run_tier1() {
    echo ""
    printf "${COLOR_BOLD}${COLOR_BLUE}>>> EXECUTING TIER 1: FEATURE COVERAGE (75 Tests across 15 Features) >>>${COLOR_RESET}\n"
    for script in "${PROJECT_ROOT}"/tests/tier1_features/*.sh; do
        if [ -n "$TARGET_FEATURE" ]; then
            local f_num
            f_num=$(echo "$TARGET_FEATURE" | grep -oE '[0-9]+' || true)
            if [ -n "$f_num" ]; then
                local feat_tag
                feat_tag=$(printf "f%02d" "$f_num")
                if [[ "$script" != *"$feat_tag"* ]]; then
                    continue
                fi
            fi
        fi

        execute_script "$script"
    done
}

# Run Tier 2
run_tier2() {
    echo ""
    printf "${COLOR_BOLD}${COLOR_BLUE}>>> EXECUTING TIER 2: BOUNDARY & CORNER CASES (75 Tests across 15 Features) >>>${COLOR_RESET}\n"
    for script in "${PROJECT_ROOT}"/tests/tier2_boundaries/*.sh; do
        if [ -n "$TARGET_FEATURE" ]; then
            local f_num
            f_num=$(echo "$TARGET_FEATURE" | grep -oE '[0-9]+' || true)
            if [ -n "$f_num" ]; then
                local feat_tag
                feat_tag=$(printf "f%02d" "$f_num")
                if [[ "$script" != *"$feat_tag"* ]]; then
                    continue
                fi
            fi
        fi

        execute_script "$script"
    done
}

# Run Tier 3
run_tier3() {
    echo ""
    printf "${COLOR_BOLD}${COLOR_BLUE}>>> EXECUTING TIER 3: CROSS-FEATURE COMBINATIONS (15 Integration Tests) >>>${COLOR_RESET}\n"
    for script in "${PROJECT_ROOT}"/tests/tier3_combinations/*.sh; do
        execute_script "$script"
    done
}

# Run Tier 4
run_tier4() {
    echo ""
    printf "${COLOR_BOLD}${COLOR_BLUE}>>> EXECUTING TIER 4: REAL-WORLD APPLICATION SCENARIOS (8 Workloads) >>>${COLOR_RESET}\n"
    for script in "${PROJECT_ROOT}"/tests/tier4_realworld/*.sh; do
        execute_script "$script"
    done
}

# Execution Dispatch
if [ "$RUN_ALL" = true ]; then
    run_tier1
    run_tier2
    run_tier3
    run_tier4
else
    case "$TARGET_TIER" in
        1) run_tier1 ;;
        2) run_tier2 ;;
        3) run_tier3 ;;
        4) run_tier4 ;;
        *)
            if [ -n "$TARGET_FEATURE" ]; then
                run_tier1
                run_tier2
            else
                echo "Invalid tier selection: $TARGET_TIER" >&2
                exit 2
            fi
            ;;
    esac
fi

# Print Aggregated Summary
echo ""
echo "=============================================================================="
printf "${COLOR_BOLD}FINAL AGGREGATED TEST SUMMARY:${COLOR_RESET}\n"
printf "  Total Tests Executed: %d\n" "$OVERALL_TOTAL"
printf "  ${COLOR_GREEN}Passed:                %d${COLOR_RESET}\n" "$OVERALL_PASSED"
printf "  ${COLOR_RED}Failed:                %d${COLOR_RESET}\n" "$OVERALL_FAILED"
printf "  ${COLOR_YELLOW}Skipped (Progressive): %d${COLOR_RESET}\n" "$OVERALL_SKIPPED"

if [ "$OVERALL_TOTAL" -gt 0 ]; then
    EFFECTIVE_TOTAL=$((OVERALL_PASSED + OVERALL_FAILED))
    if [ "$EFFECTIVE_TOTAL" -gt 0 ]; then
        PASS_RATE=$(( (OVERALL_PASSED * 100) / EFFECTIVE_TOTAL ))
        printf "  Effective Pass Rate:  %d%% (excluding progressive skips)\n" "$PASS_RATE"
    fi
fi
echo "=============================================================================="

# Generate JSON report if requested
if [ "$OUTPUT_JSON" = true ]; then
    python3 -c "
import json
report = {
    'total': $OVERALL_TOTAL,
    'passed': $OVERALL_PASSED,
    'failed': $OVERALL_FAILED,
    'skipped': $OVERALL_SKIPPED,
    'status': 'PASSED' if $OVERALL_FAILED == 0 else 'FAILED'
}
print(json.dumps(report, indent=2))
"
fi

# Exit code: 0 if no failures, 1 if any failures occurred
if [ "$OVERALL_FAILED" -gt 0 ]; then
    printf "${COLOR_RED}RESULT: FAILED (%d test failures detected)${COLOR_RESET}\n" "$OVERALL_FAILED"
    exit 1
else
    printf "${COLOR_GREEN}RESULT: ALL EXECUTED TESTS PASSED${COLOR_RESET}\n"
    exit 0
fi
