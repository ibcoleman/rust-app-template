# CLAUDE.md

Project conventions and guardrails for AI coding agents working in this repo.
Read this before writing code or running commands.

Last verified: 2026-04-23 (library targets switched to `all_crate_deps()`)

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

## CI / Bazel Remote Cache

**Cold Bazel + Rust builds in CI are expensive** (~1,500 actions, several minutes). Each GitHub Actions runner starts with an empty cache, so without a remote cache every CI run rebuilds all transitive dependencies from scratch.

**Recommended fix: BuildBuddy free tier.**

1. Sign up at [buildbuddy.io](https://buildbuddy.io) and grab an API key.
2. Add to `.bazelrc`:
   ```
   build --remote_cache=grpcs://remote.buildbuddy.io
   build --remote_header=x-buildbuddy-api-key=${BUILDBUDDY_API_KEY}
   ```
3. Add `BUILDBUDDY_API_KEY` as a GitHub Actions secret.
4. Pass it as `env: BUILDBUDDY_API_KEY: ${{ secrets.BUILDBUDDY_API_KEY }}` in each workflow job.
5. For local use, create a gitignored `.bazelrc.user` with the key hardcoded.

**Trade-offs:** External dependency (outage = cold builds, not breakage); free tier has storage/bandwidth limits; build artifacts are uploaded to a third party. Cache-only — you still compile on the runner, just skip already-cached artifacts.

This is not wired up in the template by default because it requires a per-project API key. Set it up immediately after cloning.

## Rust Conventions

- **Use `Result` + `?` for every fallible operation.** No `unwrap()` / `expect()` in production code. Tests and one-off test helpers are fine.
- **Newtype-wrap primitives that carry domain meaning:** `NoteId(Uuid)` today. As new semantically-distinct values appear (other IDs, timestamps with semantic meaning, etc.), wrap them. Never pass raw `String` or `u64` across an API boundary when the value has a name.
- **Sealed `enum`s for exhaustive domain modelling;** lean on the exhaustiveness checker. `thiserror` for error types in library/adapter code.
- **Cargo.toml is the source of truth** for third-party crates. Bazel reads it via `crate_universe` in `from_cargo` mode — never hand-write `BUILD` files for external crates. After editing Cargo.toml, repin with `just bazel-repin`. Both `Cargo.lock` and `MODULE.bazel.lock` are committed.
- **Adding a new crate — one step for library targets, two for binary/tests:**
  1. Add the crate to `Cargo.toml`, then run `just bazel-repin` to regenerate `MODULE.bazel.lock` so the crate is fetchable. The library targets `:lib` and `:lib_with_test_helpers` use `all_crate_deps(normal = True)` / `all_crate_deps(proc_macro = True)` from `@crates//:defs.bzl`, so new Cargo.toml deps flow in automatically — no BUILD.bazel edit needed for them.
  2. **If the new crate is used by `:app` or any `rust_test` in `tests/BUILD.bazel`:** add `"@crates//:<crate-name>"` to that target's explicit `deps = [...]` list. Those targets intentionally keep explicit dep lists to document exactly what they directly import. Forgetting this when required produces `unresolved import` errors at `bazel build` time even though `cargo check` succeeds.

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
- **`DATABASE_URL` is optional.** The test defaults to `postgres://app:app@localhost:5432/app`, which matches the kind-cluster postgres that Tilt port-forwards while `just dev` is running — so in the canonical flow (`just dev` in one terminal, `just test-integration` in another) you don't need to set anything. To override (e.g. point at a docker-compose postgres on a different port), export `DATABASE_URL` in your shell; the Justfile forwards it into the Bazel sandbox via `--test_env=DATABASE_URL`.
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

## Starting a New Project From This Template

Two post-clone commands get you from "template" to "empty scaffolding in house style":

1. **Rename.** `just rename <kebab-name>` — wraps `scripts/bulk-rename.sh`, which edits every tracked file containing `rust-app-template` / `rust_app_template`. It enumerates via `git ls-files | xargs grep -l ...` so new file types (added in later phases or by you) are covered automatically — no hardcoded list to drift. Lockfiles (`Cargo.lock`, `MODULE.bazel.lock`, `frontend/pnpm-lock.yaml`) are skipped; they regenerate.

2. **Strip the example domain.** `just clean-examples` — removes the `Note` CRUD example (domain, port, adapter, migration, HTTP handler, tests, frontend wiring) and keeps `Greeter` as a minimal reference of the port/adapter/handler pattern without a DB.

   Mechanism: the template marks example code with two kinds of banner:

   - `@EXAMPLE-FILE <tag>` at the top of a whole-file example. `clean-examples` deletes those files entirely.
   - `@EXAMPLE-BLOCK-START <tag>` / `@EXAMPLE-BLOCK-END <tag>` around example-specific regions inside a shared file. `clean-examples` strips the region including its markers.

   The tag (`notes`, `greeter`, etc.) is documentation — the script strips any block it finds, regardless of tag. Post-strip, `cargo fmt` runs to normalize whitespace artifacts (trailing blanks, single-field struct literals with dangling commas). After running, `just check` should pass on the empty scaffolding.

**After both:**
- Review `jj diff` (or `git diff`) before committing.
- Run `just doctor` to confirm tooling is still happy.
- `cargo machete` (if installed) will find Cargo.toml dependencies that only the removed Note domain was using; prune them manually.
- Commit, then start scaffolding your own domain.

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
