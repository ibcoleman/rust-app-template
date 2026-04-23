# rust-app-template

Single-crate Rust HTTP service template with Bazel-built Vite frontend embedded in the binary, kind+Tilt inner loop, and property + mutation testing baseline. Start here when you need a production-grade Rust service with a web UI.

## Prerequisites

- **WSL (Windows)** or **Codespaces** (GitHub)
- Or a Linux machine with: `bazelisk`, `kind`, `kubectl`, `tilt`, `pnpm`, `rust-analyzer`

The **devcontainer** (included) auto-installs everything except the shell env setup. Open in VS Code and select "Reopen in Container."

WSL users: export `ENABLE_LSP_TOOL=1` from your shell rc file (e.g., `.zshrc`).

## Use This Template

1. **Click "Use this template"** on GitHub to create your own repo.
2. **Clone your new repo locally:**
   ```bash
   git clone https://github.com/YOU/my-service
   cd my-service
   ```
3. **Rename the crate** using the provided sed script:
   ```bash
   bash scripts/bulk-rename.sh my_service my-service
   ```
   (First arg: snake_case crate name. Second arg: kebab-case repo/binary name.)

   The script edits every tracked file containing `rust-app-template` or `rust_app_template` (enumerated via `git ls-files | xargs grep -l ...`), so new file types are covered automatically. It runs `cargo fmt` at the end to normalize import ordering (renaming can shift alphabetical order of `use` statements). Lockfiles (`Cargo.lock`, `MODULE.bazel.lock`, `frontend/pnpm-lock.yaml`) and the script itself are excluded.

   Review `git diff` before committing. You may need to `just bazel-repin` and `(cd frontend && pnpm install)` afterwards to refresh lockfiles.

4. **Verify tooling:**
   ```bash
   just doctor
   ```

5. **Start the dev loop:**
   ```bash
   just dev
   ```
   This launches Tilt, which rebuilds everything on file changes.

6. **Open http://localhost:8080 in your browser.** You should see the landing page.

## Repo Structure

```
src/
  domain/          # Value types (NoteId, MAX_NOTE_BODY_LEN) — no I/O, no side effects
  ports/           # Async trait abstractions (NoteRepository) — contracts between layers
  adapters/        # Concrete impls (PgNotes) — external I/O (database, APIs)
  http/            # Axum router, handlers, error mapping, embedded frontend assets
  main.rs          # AppState wiring, startup
frontend/          # Vite + vanilla TypeScript, built by pnpm, embedded in binary
k8s/base/          # Kubernetes manifests (deployment, service, PostgreSQL statefulset)
k8s/overlays/local/ # kind overlay — local cluster config
migrations/        # sqlx migrations (numbered, applied on startup)
tests/             # API tests (:api), property tests (:properties), integration tests (:integration_db)
Tiltfile           # Local inner-loop definition (kind + Bazel watches)
Justfile           # Single human/agent entry point for common tasks
Dockerfile         # Multi-stage build (Bazel → binary)
.devcontainer/     # VS Code dev container config (auto-installs deps)
MODULE.bazel       # Bazel dependency manifest
BUILD.bazel        # Root build rules
Cargo.toml         # Rust dependencies (source of truth for Bazel via crate_universe)
```

## Core `just` Recipes

```bash
just doctor              # Verify all prerequisites are installed and on PATH
just dev                 # Tilt inner loop — kind cluster + file watches + full rebuilds on change (no HMR)
just test               # Run unit + property tests (offline, fast)
just test-integration   # Run integration tests against real PostgreSQL (slower)
just check              # cargo fmt + clippy, then bazel test //...
just mutants            # Mutation testing (nightly usage, very slow)
just bazel-repin        # Regenerate crate_universe pins after editing Cargo.toml
just reset-cluster      # Delete kind cluster, start fresh
just add-fe-dep         # Add a Node dependency to frontend/package.json
just update-fe-deps     # Refresh frontend/package-lock.json
```

## When Something Breaks

### Dev Loop Won't Start (`just dev` fails)

1. **Run `just doctor`** to check prerequisites.
   - `rust-analyzer` not found? `rustup component add rust-analyzer`
   - `bazelisk`, `kind`, `kubectl`, `tilt`, `pnpm` missing? They're in the devcontainer.
2. **Check kind context:**
   ```bash
   kubectl config current-context  # Should be "kind-kind"
   ```
3. **Look at Tilt UI logs** (localhost:10350) for clues.

### Crate pin drift

Bazel can't find a crate after editing `Cargo.toml`:
```bash
just bazel-repin
```

### Frontend build fails

```bash
cd frontend && pnpm install
```

### PostgreSQL migration fails

```bash
just reset-cluster  # Drops pgdata volume, restarts from migrations/
```

### Type errors in frontend

TypeScript strict mode is on. Check `frontend/tsconfig.json` and `frontend/README.md` for patterns.

## Next Steps

- **Read `docs/ARCHITECTURE.md`** for a hexagonal ports/adapters explainer.
- **Read `docs/ADDING-ADAPTERS.md`** for a worked example (Redis cache decorator).
- **Read `docs/RATIONALE.md`** for design decisions and Phase history (see `phase_*.md` links).
- **Understand domain layers:** Start in `src/domain/`, then `src/ports/`, then `src/adapters/`.
- **Property tests:** See `tests/properties.rs` for `proptest` patterns.

## Build + Test on CI

Every PR runs:
- `bazel build //...` — build the binary
- `bazel test //...` — unit + property tests
- `cargo fmt && cargo clippy` — linting
- Mutation testing nightly in `.github/workflows/mutants.yml`

If you hit a mutation failure, it means your tests didn't catch a real bug. Fix the test or the code.

## Embedded Frontend

The Vite frontend (TypeScript, vanilla JS, compiled assets) is built by `pnpm`, embedded in the Rust binary via `rust-embed`, and served by the HTTP adapter.

To remove the frontend (if you're building a headless API):

1. Delete `frontend/` directory
2. Remove `pnpm-workspace.yaml`
3. Edit `Cargo.toml`: remove the `[workspace]` section and the `rust-embed` dependency
4. Edit `BUILD.bazel`: remove the reference to `@npm//frontend:build`
5. Edit `src/http/mod.rs`: remove the embedded assets route and import
6. Edit `Justfile`: remove `add-fe-dep` and `update-fe-deps` recipes
7. Edit `.github/workflows/ci.yml`: remove pnpm steps
8. Remove the inline `patches:` block from `k8s/overlays/local/kustomization.yaml` (the database URL patch is frontend-agnostic and stays)

See `frontend/README.md` for details.

## License

Template is MIT. Use it as a starting point for your project.
