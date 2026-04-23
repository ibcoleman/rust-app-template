#!/usr/bin/env bash
# Rename the crate across every file that carries the template's identifiers.
# Usage: bash scripts/bulk-rename.sh <snake_case_name> <kebab-case-name>
set -euo pipefail

if [ $# -ne 2 ]; then
    echo "Usage: $0 <snake_case_name> <kebab-case-name>" >&2
    exit 1
fi

SNAKE="$1"
KEBAB="$2"

FILES=(
    Cargo.toml
    BUILD.bazel
    MODULE.bazel
    Tiltfile
    Dockerfile
    Justfile
    k8s/base/namespace.yaml
    k8s/base/deployment.yaml
    k8s/base/service.yaml
    k8s/base/postgres-statefulset.yaml
    k8s/base/postgres-service.yaml
    k8s/base/kustomization.yaml
    .devcontainer/devcontainer.json
    scripts/dev.sh
    scripts/doctor.sh
    .github/workflows/ci.yml
    .github/workflows/mutants.yml
)

for f in "${FILES[@]}"; do
    if [ -f "$f" ]; then
        sed -i.bak \
            -e "s/rust-app-template/${KEBAB}/g" \
            -e "s/rust_app_template/${SNAKE}/g" \
            "$f"
        rm -f "$f.bak"
    else
        echo "WARNING: $f not found (skipping)" >&2
    fi
done

echo "Renamed to snake='${SNAKE}', kebab='${KEBAB}'. Review git diff before committing."
