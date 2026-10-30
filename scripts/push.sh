#!/usr/bin/env bash
# ==============================================================================
# scripts/push.sh
# Ephemeral authenticated push to remote repository without credential persistence
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

TOKEN="${1:-${GITHUB_TOKEN:-}}"
if [ -z "$TOKEN" ]; then
    echo "❌ Lỗi: Chưa cung cấp GitHub Token."
    echo "   Sử dụng: ./scripts/push.sh <GITHUB_TOKEN> hoặc GITHUB_TOKEN=... ./scripts/push.sh"
    exit 1
fi

# Run secrets check before pushing
"${SCRIPT_DIR}/check-secrets.sh"

echo "🚀 [VIETKEYRA PUSH] Pushing to remote main using ephemeral authorization header..."
git -C "${PROJECT_ROOT}" -c http.extraHeader="Authorization: token ${TOKEN}" push -u origin main
echo "✔ [VIETKEYRA PUSH] Push hoàn tất thành công!"
