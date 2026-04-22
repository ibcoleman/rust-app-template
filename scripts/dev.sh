#!/usr/bin/env bash
set -euo pipefail

CLUSTER=rust-app-template

if ! kind get clusters | grep -qx "$CLUSTER"; then
    echo "Creating kind cluster '$CLUSTER'..."
    kind create cluster --name "$CLUSTER"
fi

kubectl config use-context "kind-$CLUSTER"

exec tilt up --context "kind-$CLUSTER"
