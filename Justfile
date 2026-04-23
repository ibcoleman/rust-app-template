# Verify required tooling is on PATH.
doctor:
    @bash scripts/doctor.sh

# Bring up kind cluster + Tilt. Single dev-loop entry point.
dev:
    @bash scripts/dev.sh

# Delete the kind cluster entirely.
reset-cluster:
    kind delete cluster --name rust-app-template || true

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

# Add a frontend dependency and regenerate the lockfile.
add-fe-dep pkg:
    cd frontend && pnpm add {{pkg}}
    @echo "remember: run 'just bazel-repin' if Bazel targets need updating"

# Update frontend dependencies.
update-fe-deps:
    cd frontend && pnpm update
    @echo "remember: run 'just bazel-repin' if Bazel targets need updating"
