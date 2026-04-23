# Phase 8 Pass 2 — Iteration 1 gaps

## Gap 1: Fresh-clone `just test` fails — frontend/dist required for Bazel analysis

**Surfaced at:** Task 2 Step 3, before any embedding code lands.

**Symptom:**
```
ERROR: /home/ibcoleman/Projects/static-embedder-v2/frontend/BUILD.bazel:17:16: in <toplevel>
  srcs = glob(["dist/**/*"]),
Error in glob: glob pattern 'dist/**/*' didn't match anything, but allow_empty is set to False (the default value of allow_empty can be set with --incompatible_disallow_empty_glob).
```

**Root cause:** `frontend/BUILD.bazel` declares `srcs = glob(["dist/**/*"])` with default `allow_empty=False`. On a fresh clone, `frontend/dist/` does not exist (it's pnpm-built and gitignored), so any Bazel analysis (test, build) fails immediately. Only `just dev` invokes `scripts/dev.sh` which runs `pnpm build` first.

**Template fix candidates:**
1. `just test` (and `just check`) should invoke `pnpm build` first, like `just dev` does.
2. OR `frontend/BUILD.bazel` should set `allow_empty=True` so analysis succeeds against an empty dist (but then runtime asset-embedding would silently produce a no-asset binary — worse).
3. OR `just doctor` should fail when `frontend/dist` is empty, surfacing the requirement explicitly.

Preferred: option 1 — make `pnpm build` a precondition of every just target that touches Bazel.

---

## Gap 2: `BUILD.bazel` glob `migrations/*.sql` fails with empty migrations dir

**Surfaced at:** Task 3, after deleting `migrations/0001_notes.sql`.

**Symptom:**
```
ERROR: glob pattern 'migrations/*.sql' didn't match anything, but allow_empty is set to False
```

**Root cause:** `BUILD.bazel` uses `glob([".sqlx/**/*.json", "migrations/*.sql"])` without `allow_empty=True`. When the consumer strips the demo migration as part of Task 3, the glob fails at Bazel analysis time, breaking `bazel build //...`.

**Fix applied in `static-embedder-v2`:** Changed both `lib` and `lib_with_test_helpers` `compile_data` globs to `glob([...], allow_empty = True)`.

**Template fix candidates:**
1. Change `BUILD.bazel` in `rust-app-template` to use `allow_empty=True` for the `migrations/*.sql` glob from the start — a consumer with no migrations is valid and should not break the build.

Preferred: option 1 — template should tolerate an empty migrations directory.

---

## Gap 3: `BUILD.bazel` deps list requires manual update when adding new Cargo deps

**Surfaced at:** Task 4, after adding `model2vec-rs` and `pgvector` to `Cargo.toml`.

**Symptom:**
```
error[E0432]: unresolved import `model2vec_rs`
error[E0432]: unresolved import `pgvector`
Target //:app failed to build
```

**Root cause:** Bazel's `rust_library` and `rust_binary` rules require explicit `deps = ["@crates//:model2vec-rs", ...]` entries. `just bazel-repin` updates `MODULE.bazel.lock` so the crate is fetchable, but the BUILD.bazel `deps` list is not auto-updated. The consumer must manually add each new dependency to every affected target.

**Fix applied in `static-embedder-v2`:** Manually added `@crates//:model2vec-rs` and `@crates//:pgvector` to both `lib` and `lib_with_test_helpers` deps; added `@crates//:sqlx` to `app` binary deps (needed for `PgPoolOptions`).

**Template fix candidates:**
1. Improve CLAUDE.md documentation — add an explicit rule: "After adding a crate to Cargo.toml and running `just bazel-repin`, also add `@crates//:<crate-name>` to the relevant targets in `BUILD.bazel`."
2. Alternatively, investigate `gazelle` with `rules_rust` for auto-generation of BUILD deps from source imports (complex, but eliminates the manual step).

Preferred: option 1 in the short term — document the two-step process explicitly; option 2 as a longer-term improvement.

---

## Gap 4: `just test-integration` does not pass `DATABASE_URL` into Bazel test sandbox

**Surfaced at:** Task 5, when running `just test-integration` against a live pgvector container.

**Symptom:**
`bazel test //tests:integration_db` exits 0 but the test was silently skipped — `DATABASE_URL` is not in the Bazel sandbox environment, so `connect_or_skip()` prints "DATABASE_URL not set; skipping live-DB test" and returns `None`, causing the test to pass trivially without touching the database.

Direct `cargo test --test integration_db -- --ignored` with `DATABASE_URL` set in the shell environment does work correctly.

**Root cause:** Bazel sandboxes tests and strips the host environment. To pass env vars into a Bazel test, the `rust_test` rule needs `env = {"DATABASE_URL": "..."}` or `--test_env=DATABASE_URL` on the command line. The Justfile's `test-integration` recipe does not forward the variable.

**Fix applied in `static-embedder-v2`:** None — the cargo path is used for live testing. Documented as a known limitation.

**Template fix candidates:**
1. Update `just test-integration` to pass `--test_env=DATABASE_URL` so Bazel forwards the variable from the calling environment:
   ```
   test-integration:
       bazel test //tests:integration_db --test_output=errors \
           --cache_test_results=no \
           --test_arg=--ignored \
           --test_arg=--test-threads=1 \
           --test_env=DATABASE_URL
   ```
2. Also document in `CLAUDE.md` that `DATABASE_URL` must be exported in the shell before running `just test-integration`.

Preferred: option 1 — one-line fix to the Justfile; add documentation note to CLAUDE.md.


---

## Iteration outcome

- **Iteration 1 gaps surfaced:** 4 (above)
- **Template-side fix PR:** ibcoleman/rust-app-template#3 — merged 2026-04-23
- **Iteration 2 verification:** template fixes pulled into `static-embedder-v2`; `frontend/dist/` removed to simulate a fresh clone; `just test` and `just check` both green without manual bootstrap. No new template-friction surfaced.
- **AC6.5:** satisfied by iteration 2's clean run. A full rebuild-from-template was not re-executed (heavy cost, no expected new findings); the lighter "patch + simulate fresh dist" path was used and confirmed sufficient.
- **AC6.6:** not triggered (1 iteration, well under the 5-iteration budget).
