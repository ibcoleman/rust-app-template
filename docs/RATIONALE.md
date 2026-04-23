# Architecture & Implementation Rationale

This document explains the major design choices behind `rust-app-template` and what
the template trades away in exchange for them.

## 1. Why ports and adapters

Pure domain types in the centre, side effects pushed to the edges. Services depend on
port traits, never on adapter implementations directly.

- **Types are guardrails.** A service that takes `Arc<dyn NoteRepository>` cannot reach
  for a database, an HTTP client, or the clock. The compiler enforces what the design
  intends.
- **One pattern per adapter.** A new adapter is always: define a port trait in
  `src/ports/`, implement it in `src/adapters/`, wire it into `AppState` in
  `src/http/mod.rs`. There is no second way.
- **Test seam falls out for free.** Offline tests use in-memory implementations of the
  same port trait (`tests/support/mod.rs`); integration tests use the real adapters.
  No mocking framework, no patching.

Worked examples in the demo domain: `GreetingPort` / `StaticGreeter`, and
`NoteRepository` / `PgNotes`. The template's `docs/ADDING-ADAPTERS.md` walks through
adding a new port end-to-end.

## 2. Why one-engine Bazel

Bazel (`rules_rust` + `crate_universe`) is the single canonical build for the Rust
side. Frontend is built with `pnpm` + Vite outside Bazel and consumed as a `filegroup`
(see §6 below for why the frontend isn't fully hermetic).

- **`rules_rust` reads `Cargo.toml` via `crate_universe`.** Cargo stays the source of
  truth for dependencies; we don't hand-write `BUILD` files for crates. Repinning is
  one `just bazel-repin` command.
- **`rust-embed` bakes frontend assets into the binary.** One deployable artifact,
  no separate static-file servers or volume mounts.
- **`bazel test //...` is the canonical test command.** No `cargo test` / `bazel test`
  divergence to debug. CI runs the same command a developer runs.

Trade-off: a small but real friction tax for adding a Cargo dependency
(`Cargo.toml` edit + `just bazel-repin` + manual `@crates//:foo` deps add in
`BUILD.bazel`). Documented in `CLAUDE.md`.

## 3. Why property + mutation testing

Examples are not enough. Two complementary techniques live above example tests:

- **`proptest` (per-PR).** Property tests assert invariants over generated input
  spaces — embedding length is always `EMBEDDING_DIM`, parsing then serializing is
  the identity, an inserted ID is always retrievable. Hard to write code that passes
  but is wrong.
- **`cargo-mutants` (nightly CI).** Flips operators, deletes statements, replaces
  return values. If a mutant survives, your tests didn't actually verify behaviour.
  Mutation score is treated as a first-class quality metric — a drop is a regression
  even if all example-based tests still pass.

Both are wired into CI: `just test` runs property tests; `.github/workflows/mutants.yml`
runs `cargo-mutants` on a nightly schedule.

## 4. Why one inner loop

There is one entry point for development: `just dev`. It boots a `kind` cluster,
launches Tilt, and Tilt invokes `bazel build //:app` on file changes.

What we explicitly *do not* support:

- `cargo run` (skips the Bazel build path; can succeed when Bazel fails)
- `cargo watch` / `vite dev` / `tsc --watch` (hot-reload paths that diverge from prod)
- `vite preview` / `pnpm dev`

Every developer change goes through a Docker-image round-trip into k8s. The dev loop
matches the prod loop within a build-cycle. No surprises from "works on my laptop,
breaks in CI."

Cost: dev iterations are slower than `cargo run`. The win is that "works locally"
genuinely means "works as deployed."

## 5. Why no `cargo-generate` placeholders

`cargo-generate` is the obvious-looking way to make a Rust template parameterizable.
We don't use it.

- **Author cost.** `cargo-generate` requires placeholder syntax in source files,
  template metadata, and a separate test path for the templating itself. Maintaining
  this for a single-author template is overhead with no gain.
- **Consumer cost.** A consumer must install `cargo-generate`, learn its syntax, and
  trust the template's parameterization is correct. The alternative — `git clone`
  followed by one `sed` pass — is friction-light and inspectable.
- **Renaming is rare.** A consumer renames the crate exactly once, on day zero. Any
  later renames are about the *application*, not the template, so investing in
  templating tooling for that one event is poor leverage.

The rename is automated by `scripts/bulk-rename.sh`, which enumerates files via
`git ls-files | xargs grep -l` so new file types added in later phases are covered
without code changes (see `CLAUDE.md` § "Crate-Rename Sed Checklist").

## 6. Why the frontend isn't built by Bazel

Phase 5 attempted full Bazel hermeticity for the frontend (Vite via `rules_js`
inside the Bazel sandbox). The integration friction with `rules_rust` `compile_data`
proved higher than the hermeticity benefit. Current shape:

```
frontend/src/
  └─ main.ts, api.ts, types.ts, ...
       ↓ (pnpm build)
frontend/dist/
       ↓ (Bazel filegroup //frontend:dist)
src/lib.rs (rust-embed compile_data)
       ↓
   Binary with embedded assets
```

What this costs:

- `frontend/dist/` is gitignored. A fresh clone running `bazel build //...` fails
  with `frontend/dist not found`. `just dev`, `just test`, and `just check` all
  invoke `pnpm build` first via the `fe-build` Justfile recipe (added during Phase 8
  iteration-1 gap fix).
- `rust-embed` uses `debug-embed` so debug binaries also embed assets. Without it,
  debug binaries would try to read assets from disk at runtime, breaking containerized
  workflows.
- CI must run `pnpm install --frozen-lockfile && pnpm build` before `bazel test //...`.

What we keep: the Rust side stays fully hermetic. Frontend is a pre-built dependency
of the binary, not a runtime concern.

### `must-use-result` ESLint rule

`eslint-plugin-neverthrow`'s `must-use-result` rule is disabled in
`frontend/eslint.config.js` due to incompatibility with flat config + `@typescript-eslint`
8.x. Unused-`Result` discipline is enforced at code review until the upstream rule
catches up.

## 7. Pass 2 validation: `static-embedder-v2` rebuild

The template was validated by rebuilding `ibcoleman/static-embedder` (a real
embedding service over pgvector + Model2Vec) on top of it as
`ibcoleman/static-embedder-v2`. The original `static-embedder` repo was untouched
throughout (verified by HEAD-SHA comparison before and after).

**Result: 1 gap-fix iteration to convergence** (well under the 5-iteration budget).
Iteration 1 surfaced four template-side gaps:

1. Fresh-clone `bazel build` failed because `frontend/dist/` glob had `allow_empty=False`.
2. `BUILD.bazel` `migrations/*.sql` glob had the same problem after deleting demo
   migrations.
3. Adding a Cargo dependency required a manual `BUILD.bazel` `deps` update that
   wasn't documented.
4. `just test-integration` silently skipped because `DATABASE_URL` wasn't forwarded
   into the Bazel sandbox.

All four were fixed in PR
[#3](https://github.com/ibcoleman/rust-app-template/pull/3) and verified by re-running
the full v2 verification chain against a simulated fresh clone. No new gaps surfaced
in iteration 2; AC6.5 (zero template-side changes in a final iteration) was satisfied.

The full gap log lives at `docs/implementation-plans/phase8-gaps/iteration-1.md`.

## Summary of trade-offs

| Aspect | Gain | Cost |
|---|---|---|
| Ports & adapters | Compiler-enforced isolation; no mocking framework | One-time pattern learning curve |
| One-engine Bazel | One canonical build; Cargo source-of-truth | Two-step add-a-dep workflow |
| `proptest` + `cargo-mutants` | Tests that bite | More test-authoring effort upfront |
| One inner loop (`just dev`) | Local == prod within a build cycle | Slower than `cargo run` |
| No `cargo-generate` | No templating tax for author or consumer | Manual sed (one-shot, scripted) |
| Pre-built frontend | Simple, debuggable | Not hermetic; CI bootstrap step |

This template prioritizes **clarity, reliability, and compiler-enforced correctness**
over maximal hermeticity or minimum keystrokes.
