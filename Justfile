# List available recipes (default).
default:
    @just --list

# Verify required tooling is on PATH.
doctor:
    @bash scripts/doctor.sh

# Rename the template to a new project name. Example: `just rename my-app`.
rename kebab:
    bash scripts/bulk-rename.sh $(echo {{kebab}} | tr '-' '_') {{kebab}}

# Remove the example `Note` domain (DB-backed CRUD). Keeps `Greeter` as a minimal reference.
clean-examples:
    @bash scripts/clean-examples.sh

# Bring up kind cluster + Tilt. Single dev-loop entry point.
dev:
    @bash scripts/dev.sh

# Delete the kind cluster entirely.
reset-cluster:
    kind delete cluster --name rust-app-template || true

# Tail logs from a cluster component. Run without args to list available components.
logs component="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "{{component}}" ]; then
        echo "Available components:"
        just _components
        exit 0
    fi
    kubectl logs -l app={{component}} --tail=100 -f

# Drop into a psql session against the local postgres instance.
psql:
    #!/usr/bin/env bash
    set -euo pipefail
    pod=$(kubectl get pod -l app=postgres -o name 2>/dev/null | head -1)
    if [ -z "$pod" ]; then
        echo "No postgres pod found; is 'just dev' running?" >&2
        exit 1
    fi
    kubectl exec -it "$pod" -- psql -U app -d app

# Open a shell inside a cluster component. Run without args to list available components.
shell component="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "{{component}}" ]; then
        echo "Available components:"
        just _components
        exit 0
    fi
    pod=$(kubectl get pod -l app={{component}} -o name 2>/dev/null | head -1)
    if [ -z "$pod" ]; then
        echo "No pod found with label app={{component}}" >&2
        exit 1
    fi
    kubectl exec -it "$pod" -- sh

# List `app` labels of pods in the current kubectl context (helper for `logs` / `shell`).
_components:
    @kubectl get pods -o jsonpath='{range .items[*]}{.metadata.labels.app}{"\n"}{end}' 2>/dev/null | sort -u | grep -v '^$' | sed 's/^/  /'

# Build the frontend bundle. Required before any Bazel target that consumes //frontend:dist.
fe-build:
    cd frontend && pnpm install --frozen-lockfile && pnpm build

# Offline test suite (fakes; no Postgres required).
# Note: test targets //tests:api and //tests:properties come online in Phase 2/3.
# Until then, this recipe will fail with a bazel error (expected state for Phase 1).
test: fe-build
    bazel test //tests:api //tests:properties --test_output=errors

# Integration tests that require live Postgres reachable via DATABASE_URL.
# DATABASE_URL is forwarded into the Bazel sandbox via --test_env.
test-integration: fe-build
    bazel test //tests:integration_db --test_output=errors --cache_test_results=no --test_arg=--ignored --test_arg=--test-threads=1 --test_env=DATABASE_URL

# Formatting, linting, and `just test` — matches CI.
check: fe-build
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    just test
    cd frontend && pnpm lint

# Auto-fix formatting and clippy warnings where possible.
fix:
    cargo fmt
    cargo clippy --fix --allow-dirty --allow-staged --all-targets

# Run cargo-mutants locally (nightly CI mirror).
mutants:
    cargo mutants --no-shuffle -j 2

# Regenerate crate_universe pins after editing Cargo.toml.
bazel-repin:
    CARGO_BAZEL_REPIN=1 bazel mod tidy
    bazel fetch @crates//...

# Add a Rust dependency. Must add manual BUILD.bazel step for app and rust_test.
add-dep crate *args:
    cargo add {{crate}} {{args}}
    just bazel-repin
    @echo ""
    @echo "if {{crate}} is used by :app or a rust_test target, add"
    @echo "  \"@crates//:{{crate}}\""
    @echo "to that target's deps list (see docs/ADDING-ADAPTERS.md)."

# Add a frontend dependency and regenerate the lockfile.
add-fe-dep pkg:
    cd frontend && pnpm add {{pkg}}
    @echo "remember: run 'just bazel-repin' if Bazel targets need updating"

# Update frontend dependencies.
update-fe-deps:
    cd frontend && pnpm update
    @echo "remember: run 'just bazel-repin' if Bazel targets need updating"
