# CLAUDE.md

Project conventions and guardrails for AI coding agents working in this repo.
Read this before writing code or running commands.

Last verified: 2026-04-22 (Phase 8 iteration-1 fixes: 2026-04-23)

## Status

This template is complete and ready for use. All rules below are **Enforced**.

| Rule | Status | Notes |
|---|---|---|
| `Result` + `?` for every fallible op | **Enforced** | Domain + adapters both follow this |
| No `unwrap()` / `expect()` in non-test code | **Enforced** | Clippy enforces in CI |
| Sealed `enum` domain errors via `thiserror` | **Enforced** | `NoteError`, `RepoError` |
| Newtype-wrap primitives with domain meaning | **Enforced** | `NoteId(Uuid)` in place |
| Property-based tests for non-trivial logic | **Enforced** | `tests/properties.rs` with `proptest` |
| Mutation testing in CI | **Enforced** | `.github/workflows/mutants.yml` nightly |
| Build via Bazel (`rules_rust` + `crate_universe`) | **Enforced** | `bazel build //...` + `bazel test //...` canonical |
| Local orchestration via Tilt + local k8s | **Enforced** | `just dev` wraps Tilt + kind |
| Single `just dev` path, no hot reload | **Enforced** | One entry point for all dev work |
| Pedantic TypeScript (strict, `neverthrow`, `type-fest`) | **Enforced (partial)** | Frontend strict config, `Tagged` types only; `must-use-result` disabled, see `docs/RATIONALE.md` |
| Frontend built via pnpm, not Bazel | **Enforced** | See `docs/RATIONALE.md` for Phase 5 pivots |
| LSP plugin integration | **Enforced (env)** | `ENABLE_LSP_TOOL=1`, `rust-analyzer` on PATH |

## Philosophy

- **Types are guardrails.** Push errors and intent into the type system so the compiler — not human review — catches agent mistakes. Make wrong states unrepresentable.
- **One dev path.** Every change flows through a single build engine (Bazel). No hot reloads, no parallel "fast paths" that can diverge from the real build.
- **Tests that bite.** Property-based tests make it hard to write wrong code that passes. Mutation tests make it hard to write useless tests that pass.

## Stack

- **Language:** Rust (Cargo for dependency management, Bazel for build)
- **Frontend:** TypeScript + Vite (vanilla TS scaffold, built by pnpm, embedded in binary)
- **Build:** Bazel (`rules_rust` + `crate_universe` for Rust; pnpm/Vite for the frontend (pre-built outside Bazel and consumed as a `filegroup`))
- **Local orchestration:** Tilt watching Bazel outputs, deploying into local k8s (`k3d`/`kind`)
- **Database:** PostgreSQL (migrations via `sqlx`)
- **Testing:** `proptest` (property), real PostgreSQL (integration), `cargo-mutants` (mutation)
- **Command runner:** `just`

## Dev Loop

From the repo root:

```bash
just doctor          # Verify prerequisites on PATH (rust-analyzer, bazelisk, kind, etc.)
just dev             # Tilt + local k8s inner loop; rebuilds on file changes
just test            # Unit + property tests (offline, fast)
just test-integration # Integration tests against real PostgreSQL (slower)
just check           # cargo fmt + clippy, then bazel test //...
just mutants         # Mutation testing (slow, nightly usage)
just bazel-repin     # Regenerate crate_universe pins after Cargo.toml edits
just reset-cluster   # Delete kind cluster, rebuild from scratch
just add-fe-dep      # Add a Node dependency to frontend/package.json
just update-fe-deps  # Refresh frontend/package-lock.json
```

Post-Phase 6, Bazel owns the Rust build, tests, and binary assembly; the frontend is pre-built by `pnpm` and consumed via a `filegroup`. `just dev` runs `scripts/dev.sh` which runs `pnpm build` before launching Tilt. See `docs/RATIONALE.md` for the pivot history.

Do **not** add `cargo watch`, `tsc --watch`, `vite dev`, `vite preview`, `pnpm dev`, or other hot-reload pathways.
They present a state that does not match the real build and confuse both humans and agents.

## Rust Conventions

- **Use `Result` + `?` for every fallible operation.** No `unwrap()` / `expect()` in production code. Tests and one-off test helpers are fine.
- **Newtype-wrap primitives that carry domain meaning:** `NoteId(Uuid)` today. As new semantically-distinct values appear (other IDs, timestamps with semantic meaning, etc.), wrap them. Never pass raw `String` or `u64` across an API boundary when the value has a name.
- **Sealed `enum`s for exhaustive domain modelling;** lean on the exhaustiveness checker. `thiserror` for error types in library/adapter code.
- **Cargo.toml is the source of truth** for third-party crates. Bazel reads it via `crate_universe` in `from_cargo` mode — never hand-write `BUILD` files for external crates. After editing Cargo.toml, repin with `just bazel-repin`. Both `Cargo.lock` and `MODULE.bazel.lock` are committed.
- **Adding a new crate is a two-step Bazel update:**
  1. Add the crate to `Cargo.toml`, then run `just bazel-repin` to regenerate `MODULE.bazel.lock` so the crate is fetchable.
  2. Add `"@crates//:<crate-name>"` to the `deps = [...]` list of every Bazel target that uses it (typically `:lib`, `:lib_with_test_helpers`, `:app`, and any test target). `crate_universe` makes the crate *available* but does not auto-wire it into target dep lists. Forgetting step 2 produces `unresolved import` errors at `bazel build` time even though `cargo check` succeeds.

## TypeScript Conventions

### tsconfig (non-negotiable)

```json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "noFallthroughCasesInSwitch": true
  }
}
```

### Error Handling with `neverthrow`

- Return `Result` / `ResultAsync`. **Do not `throw`** in application code.
- `eslint-plugin-neverthrow`'s plugin is registered, but the `must-use-result` rule is currently disabled (tooling incompatibility with flat config + @typescript-eslint 8.x — see `docs/RATIONALE.md`). Unused-`Result` handling is enforced at code review until the rule can be re-enabled. Do not silently drop `Result` values.
- `try` / `catch` is reserved for bridging to libraries that throw. Wrap those boundaries into `Result` at the seam and keep the rest of the code in `Result`-land.

### `type-fest` — Nominal Types

- **`Tagged`** — for nominal types. `UserId` is not assignable to `OrderId` even though both wrap `string`. Use for every ID-like field.
- **`Except`, `SetRequired`, `SetOptional`** — derive related types without duplication.
- **`ReadonlyDeep`** — TS `readonly` is shallow and structural. Use when you want Java-`final`-like guarantees.

## Testing

### Property-Based Tests (required for non-trivial logic)

- Rust: `proptest` — in place. See `tests/properties.rs`.
- Example: `StaticGreeter` invariants verify the greeting never exceeds domain constraints.

Anything with a non-trivial input space — parsers, state machines, serializers, domain logic with invariants — gets properties, not just examples.

### Integration Tests (real database)

- Tag with `#[ignore]` and Bazel `manual` tag.
- Run via `just test-integration` or `bazel test //tests:integration_db --config=live` against docker-compose PostgreSQL.
- **Required env:** `DATABASE_URL` must be exported in your shell before invoking `just test-integration`. The Justfile forwards it into the Bazel sandbox via `--test_env=DATABASE_URL`; without it set, the test silently skips (it cannot reach the DB from inside the sandbox).
- Use real DB engines, not in-memory substitutes.

### Mutation Testing (CI, nightly)

- Rust: `cargo-mutants` — runs nightly in `.github/workflows/mutants.yml`.
- **Mutation score is a first-class quality metric.** If a change drops the mutation score, it is a regression — even if every example-based test still passes.

## LSP / Agent Tooling

Code-intelligence tools (goto-def, find-refs, type-at-cursor) are exposed to the agent via Claude Code's LSP integration.

Setup (target state):

1. **`ENABLE_LSP_TOOL=1`** in the dev environment. The devcontainer sets this via `remoteEnv`; WSL users should export it from their shell rc.
2. **Language server binaries on PATH:**
   - `rust-analyzer` — `rustup component add rust-analyzer`.
   - `vtsls` — when TS automation is needed (frontend type-checking).
3. **In Claude Code:** `/plugin marketplace add anthropics/claude-plugins-official`, then `/plugin install` the Rust code-intelligence plugin.
4. **Restart Claude Code.** Confirm with `claude --debug` — look for `LSP server plugin:rust:rust initialized`.

`just doctor` verifies steps 1 and 2.

## Crate-Rename Sed Checklist

When you clone this template and want to rename the crate:

1. **Use the canonical script:**
   ```bash
   bash scripts/bulk-rename.sh <snake_case_name> <kebab-case-name>
   ```

2. **The script edits every tracked file** that contains `rust-app-template` or `rust_app_template`. It enumerates via `git ls-files | xargs grep -l ...` so new file types (added in later phases or by you) are covered automatically — no hardcoded list to drift.

   Files excluded from rewriting: the script itself, `Cargo.lock`, `MODULE.bazel.lock`, `frontend/pnpm-lock.yaml` (lockfiles are regenerated by tooling).

3. **After running the script:**
   - Review `git diff` to verify all names changed correctly.
   - Run `just doctor` to ensure nothing is broken.
   - Commit with a message like `chore: rename crate to my-service`.

## Rules for the Agent

**Do:**

- Route build/deploy through `just` targets.
- Model every fallible operation as `Result`. Propagate with `?`.
- Use newtype wrappers for IDs and other semantically-distinct values.
- Add property-based tests for any non-trivial new logic.
- Run `just doctor` if anything about the toolchain feels off.

**Don't:**

- Add hot-reload or dev-server paths (`cargo watch`, `tsc --watch`, `vite dev`, `vite preview`, `pnpm dev`). `just dev` is the only dev-loop entry point.
- `throw` in TS application code. Don't `unwrap()` / `expect()` in Rust production code.
- Silence the type checker (`any`, non-trivial `as` casts, `// @ts-ignore`).
- Use raw `string` / number primitives for IDs — wrap them.
- Ship tests that only exercise the happy path. If `cargo-mutants` can flip a comparison and your tests still pass, the tests are not doing their job.
- Hand-write BUILD files for third-party crates. Use `crate_universe`; regenerate with `just bazel-repin` after Cargo.toml edits.
