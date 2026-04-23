#!/usr/bin/env bash
# Verify all prerequisites are on PATH. Reports ok/MISSING for every tool
# before exiting, so you see all gaps at once rather than fixing one at a time.
set -euo pipefail

missing=0

require() {
    local cmd="$1" hint="$2"
    if command -v "$cmd" >/dev/null 2>&1; then
        echo "ok       $cmd"
    else
        echo "MISSING  $cmd  ($hint)"
        missing=1
    fi
}

warn() {
    local cmd="$1" hint="$2"
    if command -v "$cmd" >/dev/null 2>&1; then
        echo "ok       $cmd"
    else
        echo "warn     $cmd  ($hint)"
    fi
}

require bazel      "install bazelisk: brew install bazelisk or https://github.com/bazelbuild/bazelisk"
require kind       "install kind: https://kind.sigs.k8s.io/docs/user/quick-start/#installation"
require kubectl    "install kubectl: https://kubernetes.io/docs/tasks/tools/"
require tilt       "install tilt: https://docs.tilt.dev/install.html"
require pnpm       "install pnpm: npm install -g pnpm"
require docker     "install Docker Engine or Docker Desktop"
require sqlx       "cargo install sqlx-cli --no-default-features --features postgres"

warn rust-analyzer "rustup component add rust-analyzer"

if ! docker compose version >/dev/null 2>&1; then
    echo "MISSING  docker compose plugin  (install Docker Desktop or the compose plugin)"
    missing=1
else
    echo "ok       docker compose"
fi

if [ "${ENABLE_LSP_TOOL:-}" != "1" ]; then
    echo "warn     ENABLE_LSP_TOOL not set to 1 (export from shell rc or devcontainer remoteEnv — see CLAUDE.md)"
else
    echo "ok       ENABLE_LSP_TOOL"
fi

if [ "$missing" -ne 0 ]; then
    echo ""
    echo "FAIL: one or more required tools are missing"
    exit 1
fi

echo ""
echo "doctor: OK"
