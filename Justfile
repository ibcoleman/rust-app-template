# Verify required tooling is on PATH.
doctor:
    @bash scripts/doctor.sh

# Bring up kind cluster + Tilt. Single dev-loop entry point.
dev:
    @bash scripts/dev.sh

# Delete the kind cluster entirely.
reset-cluster:
    kind delete cluster --name rust-app-template || true

# Offline test suite (fakes; no Postgres required).
# Note: test targets //tests:api and //tests:properties come online in Phase 2/3.
# Until then, this recipe will fail with a bazel error (expected state for Phase 1).
test:
    bazel test //tests:api //tests:properties --test_output=errors

# Integration tests that require `just dev` to be running (real kind-hosted Postgres).
test-integration:
    bazel test //tests:integration_db --test_output=errors --cache_test_results=no

# Formatting, linting, and `just test` — matches CI.
check:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    just test
    # `bazel run //frontend:lint` is added in Phase 5.

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

# Add a frontend dependency (Phase 5).
add-fe-dep pkg:
    @echo "add-fe-dep {{pkg}} — implemented in Phase 5"

# Update frontend dependencies (Phase 5).
update-fe-deps:
    @echo "implemented in Phase 5"
