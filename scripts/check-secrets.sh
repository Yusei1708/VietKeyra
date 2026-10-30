#!/usr/bin/env bash
# ==============================================================================
# scripts/check-secrets.sh
# Automated Secret Scanner for VietKeyra
# Scans repository files, staged changes, and commit history for leaked secrets.
# ==============================================================================

set -euo pipefail

PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "$PROJECT_ROOT" ]; then
    PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi

echo "🔍 [VIETKEYRA SECRET SCANNER] Scanning for potential secret leakage..."

LEAKS_FOUND=0

# Secret Patterns to Block
FORBIDDEN_FILE_PATTERNS=(
    "^\.agents(/|$)"
    "^ORIGINAL_REQUEST\.md$"
    "\.token$"
    "^\.env"
    "auth\.json$"
)

# Dynamically retrieve prompt token for verification if available in un-tracked agent metadata
PROMPT_TOKEN=""
if [ -f "${PROJECT_ROOT}/.agents/teamwork/ORIGINAL_REQUEST.md" ]; then
    PROMPT_TOKEN=$(grep -oE "ghp_[A-Za-z0-9]{36}" "${PROJECT_ROOT}/.agents/teamwork/ORIGINAL_REQUEST.md" 2>/dev/null | head -n1 || true)
fi

# 1. Check Git Remote URL if Git is initialized
if [ -d "${PROJECT_ROOT}/.git" ]; then
    REMOTE_URL=$(git -C "${PROJECT_ROOT}" remote get-url origin 2>/dev/null || true)
    if [ -n "$REMOTE_URL" ]; then
        if [[ "$REMOTE_URL" == *"@"* ]] || [[ "$REMOTE_URL" == *"ghp_"* ]] || [[ "$REMOTE_URL" == *"github_pat_"* ]]; then
            echo "❌ [ERROR] Plain-text credentials or tokens detected in Git remote URL: $REMOTE_URL"
            LEAKS_FOUND=$((LEAKS_FOUND + 1))
        fi
    fi

    # Check .git/config directly for any plain-text token
    if [ -f "${PROJECT_ROOT}/.git/config" ]; then
        if grep -E "ghp_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{82}" "${PROJECT_ROOT}/.git/config" >/dev/null 2>&1; then
            echo "❌ [ERROR] Token string detected inside .git/config!"
            LEAKS_FOUND=$((LEAKS_FOUND + 1))
        fi
    fi
fi

# 2. Check Tracked Files in Git
if [ -d "${PROJECT_ROOT}/.git" ]; then
    TRACKED_FILES=$(git -C "${PROJECT_ROOT}" ls-files 2>/dev/null || true)
    for pattern in "${FORBIDDEN_FILE_PATTERNS[@]}"; do
        MATCHED_TRACKED=$(echo "$TRACKED_FILES" | grep -E "$pattern" || true)
        if [ -n "$MATCHED_TRACKED" ]; then
            echo "❌ [ERROR] Forbidden file tracked by Git:"
            echo "$MATCHED_TRACKED" | sed 's/^/    /'
            LEAKS_FOUND=$((LEAKS_FOUND + 1))
        fi
    done

    # 3. Check Staged Files
    STAGED_FILES=$(git -C "${PROJECT_ROOT}" diff --cached --name-only 2>/dev/null || true)
    for pattern in "${FORBIDDEN_FILE_PATTERNS[@]}"; do
        MATCHED_STAGED=$(echo "$STAGED_FILES" | grep -E "$pattern" || true)
        if [ -n "$MATCHED_STAGED" ]; then
            echo "❌ [ERROR] Forbidden file staged for commit:"
            echo "$MATCHED_STAGED" | sed 's/^/    /'
            LEAKS_FOUND=$((LEAKS_FOUND + 1))
        fi
    done

    # 4. Check Staged Diffs for Secret Patterns (excluding self-definition diffs)
    if [ -n "$STAGED_FILES" ]; then
        STAGED_DIFF=$(git -C "${PROJECT_ROOT}" diff --cached -U0 --text 2>/dev/null || true)
        DIFF_FILTERED=$(echo "$STAGED_DIFF" | awk '
            /^diff --git a\/(scripts|tests)\// { skip=1; next }
            /^diff --git a\/TEST_INFRA\.md/ { skip=1; next }
            /^diff --git/ { skip=0 }
            !skip { print }
        ')
        if [ -n "$DIFF_FILTERED" ]; then
            if echo "$DIFF_FILTERED" | grep -E "ghp_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{82}" >/dev/null 2>&1; then
                echo "❌ [ERROR] GitHub token pattern detected in staged changes!"
                LEAKS_FOUND=$((LEAKS_FOUND + 1))
            fi
            if [ -n "$PROMPT_TOKEN" ] && echo "$DIFF_FILTERED" | grep -F "$PROMPT_TOKEN" >/dev/null 2>&1; then
                echo "❌ [ERROR] Authoritative prompt token detected in staged changes!"
                LEAKS_FOUND=$((LEAKS_FOUND + 1))
            fi
            if echo "$DIFF_FILTERED" | grep -iE "(api[_-]?key|secret[_-]?key|password)\s*[:=]\s*['\"][A-Za-z0-9_\-]{16,}['\"]" >/dev/null 2>&1; then
                echo "❌ [ERROR] Potential API key or password literal detected in staged changes!"
                LEAKS_FOUND=$((LEAKS_FOUND + 1))
            fi
        fi
    fi

    # 5. Check Git Commit History if any commits exist
    if git -C "${PROJECT_ROOT}" rev-parse --verify HEAD >/dev/null 2>&1; then
        COMMIT_LOG=$(git -C "${PROJECT_ROOT}" log -p -n 100 --text 2>/dev/null || true)
        COMMIT_FILTERED=$(echo "$COMMIT_LOG" | awk '
            /^diff --git a\/(scripts|tests)\// { skip=1; next }
            /^diff --git a\/TEST_INFRA\.md/ { skip=1; next }
            /^diff --git/ { skip=0 }
            !skip { print }
        ')
        if echo "$COMMIT_FILTERED" | grep -E "ghp_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{82}" >/dev/null 2>&1; then
            echo "❌ [ERROR] GitHub token pattern detected in git commit history!"
            LEAKS_FOUND=$((LEAKS_FOUND + 1))
        fi
        if [ -n "$PROMPT_TOKEN" ] && echo "$COMMIT_FILTERED" | grep -F "$PROMPT_TOKEN" >/dev/null 2>&1; then
            echo "❌ [ERROR] Authoritative prompt token detected in git commit history!"
            LEAKS_FOUND=$((LEAKS_FOUND + 1))
        fi
    fi
fi

# 6. Check Non-Ignored Files on Disk
if [ -d "${PROJECT_ROOT}/.git" ]; then
    SCAN_TARGETS=$(git -C "${PROJECT_ROOT}" ls-files --exclude-standard -c -o 2>/dev/null || true)
else
    SCAN_TARGETS=$(find "${PROJECT_ROOT}" -maxdepth 4 \
        -not -path '*/.*' \
        -not -path '*/target*' \
        -not -path '*/backups*' \
        -not -path '*/ORIGINAL_REQUEST.md*' \
        -type f 2>/dev/null || true)
fi

while IFS= read -r file; do
    [ -z "$file" ] && continue
    if [[ "$file" != /* ]]; then
        full_path="${PROJECT_ROOT}/${file}"
    else
        full_path="$file"
    fi
    [ ! -f "$full_path" ] && continue

    # Skip security definition files and test files to prevent self-detection of regex patterns
    if [[ "$full_path" == *"/scripts/check-secrets.sh"* ]] || [[ "$full_path" == *"/scripts/pre-commit.sh"* ]] || [[ "$full_path" == *"/tests/"* ]]; then
        continue
    fi

    if [ -n "$PROMPT_TOKEN" ] && grep -F "$PROMPT_TOKEN" "$full_path" >/dev/null 2>&1; then
        echo "❌ [ERROR] Secret token found in file: $file"
        LEAKS_FOUND=$((LEAKS_FOUND + 1))
    fi
    if grep -E "ghp_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{82}" "$full_path" >/dev/null 2>&1; then
        echo "❌ [ERROR] Token pattern found in file: $file"
        LEAKS_FOUND=$((LEAKS_FOUND + 1))
    fi
done <<< "$SCAN_TARGETS"

if [ "$LEAKS_FOUND" -gt 0 ]; then
    echo "❌ [VIETKEYRA SECRET SCANNER] FAILED: $LEAKS_FOUND potential secret leak(s) detected!"
    exit 1
fi

echo "✅ [VIETKEYRA SECRET SCANNER] PASSED: Working tree and repository are clean of secrets."
exit 0
