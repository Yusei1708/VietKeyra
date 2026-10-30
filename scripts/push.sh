#!/usr/bin/env bash
# ==============================================================================
# scripts/push.sh
# Ephemeral authenticated push to remote repository without credential persistence
# ==============================================================================

set -euo pipefail

PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "$PROJECT_ROOT" ]; then
    PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi

TOKEN="${1:-${GITHUB_TOKEN:-}}"
if [ -z "$TOKEN" ]; then
    echo "❌ Lỗi: Chưa cung cấp GitHub Token."
    echo "   Sử dụng: ./scripts/push.sh <GITHUB_TOKEN> hoặc GITHUB_TOKEN=... ./scripts/push.sh"
    exit 1
fi

# Run secrets check before pushing
"${PROJECT_ROOT}/scripts/check-secrets.sh"

echo "🚀 [VIETKEYRA PUSH] Pushing to remote main using ephemeral authorization header..."
B64_AUTH=$(printf "Yusei1708:%s" "$TOKEN" | base64)
git -C "${PROJECT_ROOT}" -c http.extraHeader="Authorization: Basic ${B64_AUTH}" -c credential.helper="" push -u origin main
echo "✔ [VIETKEYRA PUSH] Push hoàn tất thành công!"
