#!/usr/bin/env bash
# Verify all prerequisites are on PATH and exit non-zero with an actionable
# message naming the first missing tool.
set -euo pipefail

fail() {
    echo "ERROR: $1" >&2
    exit 1
}

check_cmd() {
    command -v "$1" >/dev/null 2>&1 || fail "Missing '$1' on PATH. Install it before running 'just dev'."
}

check_cmd bazel
check_cmd kind
check_cmd kubectl
check_cmd tilt
check_cmd rust-analyzer
check_cmd pnpm
check_cmd docker

if [ "${ENABLE_LSP_TOOL:-}" != "1" ]; then
    fail "ENABLE_LSP_TOOL must be set to 1 (export it from your shell rc or devcontainer remoteEnv)."
fi

echo "doctor: OK"
