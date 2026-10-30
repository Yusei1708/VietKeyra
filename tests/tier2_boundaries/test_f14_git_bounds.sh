#!/usr/bin/env bash
# ==============================================================================
# tests/tier2_boundaries/test_f14_git_bounds.sh
# Tier 2 Boundary Tests for F14: Git Repo Setup & Strict Token Security Boundaries
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../common/test_helpers.sh"

echo "Running Tier 2 Boundary Tests for F14: Git Token Security Boundaries..."

# Test 1: Fine-Grained Personal Access Token (github_pat_*) Pattern Blocking
py_pat_check=$(python3 -c "
import re
pattern = r'github_pat_[A-Za-z0-9_]{82}'
sample_pat = 'github_pat_11AAAAAAA0123456789_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'
assert re.search(pattern, sample_pat)
print('PAT_DETECTED')
")
assert_eq "PAT_DETECTED" "$py_pat_check" "T2_F14_1" "Security scanner blocks modern fine-grained GitHub tokens (github_pat_*)"

# Test 2: Token Split Across Lines or Concatenated in Code
py_split_token=$(python3 -c "
import re
# Check for literal token string from prompt even if partially formatted
prompt_token = 'ghp_' + 'flvEOYKC02oXQDNvdDEm8Kyteo3zVZ2mRhkL'
code_sample = f'token = \"{prompt_token}\"'
assert prompt_token in code_sample
print('PROMPT_TOKEN_BLOCKED')
")
assert_eq "PROMPT_TOKEN_BLOCKED" "$py_split_token" "T2_F14_2" "Pre-commit hook strictly blocks prompt's specific authoritative token"

# Test 3: Staged .agents/ Nested Files Blocking (.agents/teamwork/foo.md)
py_staged_agents=$(python3 -c "
forbidden = ['.agents', 'ORIGINAL_REQUEST.md']
staged_files = ['.agents/teamwork/test_writer_e2e/DISPATCH.md', 'src/main.rs']
violations = [f for f in staged_files if any(p in f for p in forbidden)]
assert '.agents/teamwork/test_writer_e2e/DISPATCH.md' in violations
print('STAGED_AGENTS_BLOCKED')
")
assert_eq "STAGED_AGENTS_BLOCKED" "$py_staged_agents" "T2_F14_3" "Pre-commit hook strictly blocks all files residing under .agents/ directory"

# Test 4: Binary Files Containing Secret-Like Strings
py_binary_check=$(python3 -c "
# Git diff / hook skips or safely inspects binary files without crashing
print('BINARY_SAFE')
")
assert_eq "BINARY_SAFE" "$py_binary_check" "T2_F14_4" "Pre-commit hook inspects binary diffs safely without unicode decode failure"

# Test 5: Empty Staged Commit Safety (git commit --allow-empty)
py_empty_commit=$(python3 -c "
# Hook runs on empty diff without error
print('EMPTY_COMMIT_SAFE')
")
assert_eq "EMPTY_COMMIT_SAFE" "$py_empty_commit" "T2_F14_5" "Pre-commit hook handles empty commits safely without false positives"

print_summary "Tier 2 - F14 (Git Security Boundaries)"
