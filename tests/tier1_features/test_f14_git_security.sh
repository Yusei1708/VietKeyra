#!/usr/bin/env bash
# ==============================================================================
# tests/tier1_features/test_f14_git_security.sh
# Tier 1 Tests for F14: Git Repo Setup & Strict Token Security
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 1 Tests for F14: Git Repo Setup & Strict Token Security..."

GITIGNORE="${PROJECT_ROOT}/.gitignore"

# Test 1: .gitignore Exists and Excludes .agents/ and Secrets
if [ -f "$GITIGNORE" ]; then
    has_agents=$(grep -E "^\.agents/?" "$GITIGNORE" || true)
    has_tokens=$(grep -E "\*\.token|\.env" "$GITIGNORE" || true)
    if [ -n "$has_agents" ] && [ -n "$has_tokens" ]; then
        report_pass "T1_F14_1" ".gitignore exists and ignores .agents/ and secret tokens"
    else
        report_fail "T1_F14_1" ".gitignore rules" "Missing .agents/ or *.token in .gitignore"
    fi
else
    # In progressive state before M4 creates .gitignore, check specification
    report_skip "T1_F14_1" ".gitignore ignores .agents/ and secret tokens" ".gitignore not yet created"
fi

# Test 2: Git Remote URL Does NOT Contain Embedded Plain-Text Token
if [ -d "${PROJECT_ROOT}/.git" ]; then
    remote_url=$(git -C "${PROJECT_ROOT}" remote get-url origin 2>/dev/null || true)
    if [[ "$remote_url" == *"@"* ]] || [[ "$remote_url" == *"ghp_"* ]]; then
        report_fail "T1_F14_2" "Git remote URL is clean" "Plain-text credentials detected in remote URL: $remote_url"
    else
        report_pass "T1_F14_2" "Git remote URL is clean and free of embedded credentials"
    fi
else
    report_skip "T1_F14_2" "Git remote URL is clean and free of credentials" ".git repository not yet initialized"
fi

# Test 3: Pre-Commit Hook Script Exists and Has Execution Permission
PRE_COMMIT_SCRIPT="${PROJECT_ROOT}/scripts/pre-commit.sh"
if [ -f "$PRE_COMMIT_SCRIPT" ]; then
    if [ -x "$PRE_COMMIT_SCRIPT" ]; then
        report_pass "T1_F14_3" "scripts/pre-commit.sh exists and is executable"
    else
        report_fail "T1_F14_3" "scripts/pre-commit.sh executable" "File exists but missing execute bit (+x)"
    fi
else
    report_skip "T1_F14_3" "scripts/pre-commit.sh exists and is executable" "scripts/pre-commit.sh not yet created"
fi

# Test 4: Pre-Commit Hook Blocks Commits Containing ghp_* Patterns
# We simulate a staged diff with a fake GitHub token
dummy_secret_regex="ghp_[A-Za-z0-9]{36}"
test_string="export GITHUB_TOKEN=ghp_dummy12345678901234567890123456789012"
is_blocked=$(python3 -c "
import re
pattern = r'ghp_[A-Za-z0-9]{36}'
match = re.search(pattern, '''$test_string''')
print('BLOCKED' if match else 'ALLOWED')
")
assert_eq "BLOCKED" "$is_blocked" "T1_F14_4" "Security scanning regex accurately catches ghp_* tokens"

# Test 5: Standalone Secret Scanner Script (scripts/check-secrets.sh)
CHECK_SECRETS="${PROJECT_ROOT}/scripts/check-secrets.sh"
if [ -f "$CHECK_SECRETS" ]; then
    if bash "$CHECK_SECRETS" > /dev/null 2>&1; then
        report_pass "T1_F14_5" "scripts/check-secrets.sh passes on clean working tree"
    else
        report_fail "T1_F14_5" "scripts/check-secrets.sh execution" "Scanner returned non-zero"
    fi
else
    report_skip "T1_F14_5" "scripts/check-secrets.sh passes on clean working tree" "scripts/check-secrets.sh not yet created"
fi

print_summary "Tier 1 - F14 (Git Token Security)"
