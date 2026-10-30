#!/usr/bin/env bash
# ==============================================================================
# scripts/pre-commit.sh
# Git Pre-Commit Security Hook for VietKeyra
# Runs before every commit to ensure zero credentials, tokens, or private agent
# directories are staged into the repository.
# ==============================================================================

set -euo pipefail

PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "$PROJECT_ROOT" ]; then
    PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi

echo "🛡️  [VIETKEYRA PRE-COMMIT HOOK] Verifying staging area security..."

# 1. Deny staged blacklisted files
FORBIDDEN_FILES=(".agents" "ORIGINAL_REQUEST.md" ".env" ".token" "auth.json")
for pattern in "${FORBIDDEN_FILES[@]}"; do
    if git -C "${PROJECT_ROOT}" diff --cached --name-only | grep -E "(^|/)${pattern}" > /dev/null; then
        echo "❌ [VIETKEYRA PRE-COMMIT HOOK] LỖI BẢO MẬT: Phát hiện file cấm được stage: $pattern"
        echo "   Tuyệt đối không commit file cấu hình cá nhân hoặc dữ liệu agent!"
        exit 1
    fi
done

# 2. Deny secret patterns in staged diff
DIFF_CONTENT=$(git -C "${PROJECT_ROOT}" diff --cached -U0 --text 2>/dev/null || git -C "${PROJECT_ROOT}" diff --cached -U0 || true)

DIFF_FILTERED=$(echo "$DIFF_CONTENT" | awk '
    /^diff --git a\/(scripts|tests)\// { skip=1; next }
    /^diff --git a\/TEST_INFRA\.md/ { skip=1; next }
    /^diff --git/ { skip=0 }
    !skip { print }
')

if [ -n "$DIFF_FILTERED" ]; then
    # Check for classic GitHub Personal Access Tokens (ghp_*)
    if echo "$DIFF_FILTERED" | grep -E "ghp_[A-Za-z0-9]{36}" > /dev/null; then
        echo "❌ [VIETKEYRA PRE-COMMIT HOOK] LỖI BẢO MẬT: Phát hiện GitHub Token (ghp_...) trong nội dung commit!"
        exit 1
    fi

    # Check for fine-grained GitHub Personal Access Tokens (github_pat_*)
    if echo "$DIFF_FILTERED" | grep -E "github_pat_[A-Za-z0-9_]{82}" > /dev/null; then
        echo "❌ [VIETKEYRA PRE-COMMIT HOOK] LỖI BẢO MẬT: Phát hiện GitHub Fine-Grained PAT trong nội dung commit!"
        exit 1
    fi

    # Check for specific prompt token if present in agent transcripts
    PROMPT_TOKEN=""
    if [ -f "${PROJECT_ROOT}/.agents/teamwork/ORIGINAL_REQUEST.md" ]; then
        PROMPT_TOKEN=$(grep -oE "ghp_[A-Za-z0-9]{36}" "${PROJECT_ROOT}/.agents/teamwork/ORIGINAL_REQUEST.md" 2>/dev/null | head -n1 || true)
    fi
    if [ -n "$PROMPT_TOKEN" ] && echo "$DIFF_FILTERED" | grep -F "$PROMPT_TOKEN" > /dev/null; then
        echo "❌ [VIETKEYRA PRE-COMMIT HOOK] LỖI BẢO MẬT: Phát hiện token bài toán trong nội dung commit!"
        exit 1
    fi

    # Check for generic secret keywords with quotes
    if echo "$DIFF_FILTERED" | grep -iE "(api[_-]?key|secret[_-]?key|password)\s*[:=]\s*['\"][A-Za-z0-9_\-]{16,}['\"]" > /dev/null; then
        echo "❌ [VIETKEYRA PRE-COMMIT HOOK] LỖI BẢO MẬT: Phát hiện chuỗi API key / password có thể bị lộ!"
        exit 1
    fi
fi

# Run the comprehensive scanner
"${PROJECT_ROOT}/scripts/check-secrets.sh"

echo "✅ [VIETKEYRA PRE-COMMIT HOOK] Kiểm tra an toàn mã nguồn: HỢP LỆ."
exit 0
