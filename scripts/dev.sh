#!/usr/bin/env bash
set -euo pipefail

CLUSTER=rust-app-template

# Ensure frontend is built before bringing up Tilt.
# This is critical for fresh clones where dist/ may not exist.
echo "Building frontend..."
(cd frontend && pnpm install --frozen-lockfile && pnpm build)

if ! kind get clusters | grep -qx "$CLUSTER"; then
    echo "Creating kind cluster '$CLUSTER'..."
    kind create cluster --name "$CLUSTER"
fi

kubectl config use-context "kind-$CLUSTER"

exec tilt up --context "kind-$CLUSTER"
