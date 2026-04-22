# Architecture & Implementation Rationale

## Phase 5: Frontend Integration & Build Architecture Pivot

### The Pivot: From Hermetic Bazel Builds to Pre-Built Frontend

**Original approach (attempted):** Integrate `rules_js` Vite build into Bazel via `vite_build` rule, output via `copy_to_directory`, consumed by `rust-embed` at compile time.

**Reality:** `rules_js` maturity and `vite_build` integration with `rules_rust` compile_data proved difficult. The tooling exists but integration friction was higher than acceptable for Phase 5.

**Solution:** Pre-build frontend outside Bazel; Bazel consumes the output via a `filegroup`.

### Architecture Overview

```
frontend/src/
  └─ main.ts, api.ts, types.ts, ...
       ↓ (pnpm build)
frontend/dist/
  ├─ index.html
  ├─ assets/main.*.js
  └─ assets/*.css
       ↓ (Bazel filegroup)
//frontend:dist filegroup
       ↓ (Rust compile_data)
src/lib.rs + src/main.rs (via rust_library and rust_binary rules)
  ├─ rust_library(compile_data = ["//frontend:dist"])
  └─ rust_binary(compile_data = ["//frontend:dist"])
       ↓
   Binary with embedded assets
```

### Implications

#### 1. Hermeticity Trade-off
- **Lost:** Frontend build is not hermetic within Bazel. It relies on external state (pnpm, node_modules).
- **Retained:** Rust side remains fully hermetic. Bazel controls all Rust dependencies and compilation.
- **Justification:** Rust is the "real" build; frontend is a dependency. The cost of full JS hermeticity in Bazel outweighs the benefit.

#### 2. Fresh-Clone Workflow

When cloning the repository on a fresh machine:

```bash
git clone <repo>
cd rust-app-template

# This will fail because frontend/dist is not committed:
bazel build //:app  # ERROR: frontend/dist not found at compile time

# Solution: build frontend first, then Bazel can proceed
cd frontend && pnpm install --frozen-lockfile && pnpm build && cd ..
bazel build //:app  # SUCCESS
```

The absence of `frontend/dist` on fresh clone is **intentional** and **catches misconfigurations early**.

#### 3. CI Build Step

To bootstrap CI, `frontend/dist` must be built before running Bazel tests:

```yaml
# .github/workflows/ci.yml
steps:
  - uses: pnpm/action-setup@v4
    with:
      version: 10
  - uses: actions/setup-node@v4
    with:
      node-version: 22
  - run: pnpm install --frozen-lockfile
  - run: pnpm -F rust-app-template-frontend build
  - run: bazel test //... --test_output=errors
```

This mirrors the fresh-clone workflow and makes CI requirements explicit.

#### 4. Local Development (Tilt)

`scripts/dev.sh` runs pnpm build before Tilt up:

```bash
pnpm install --frozen-lockfile >/dev/null 2>&1 && \
pnpm build >/dev/null 2>&1 && \
tilt up
```

Tiltfile watches `frontend/src/`, `frontend/index.html`, `frontend/vite.config.ts` and reruns pnpm build on changes before rebuilding the Bazel binary.

#### 5. Compile-Time Embedding

The Rust side uses `rust-embed` with `debug-embed` feature enabled:

```rust
#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
#[debug_embed]  // Include files in debug builds too
pub struct Assets;
```

**Why `debug-embed`:** Without it, debug binaries would attempt to read from disk at runtime, breaking containerized workflows where the filesystem layout differs. Embedded assets are always available regardless of binary mode.

**Compile-time guarantee:** The `rust-embed` proc-macro expands at library compile time (in `lib.rs`, not just `main.rs`). It verifies the folder exists during compilation. If `frontend/dist/` is missing, the build fails with:

```
error: Could not read folder at path...
```

This is **intentional design**: catch misconfiguration before linking, not at runtime.

### Why `compile_data` on Both `rust_library` and `rust_binary`

- `rust_library(compile_data = ["//frontend:dist"])` (in src/lib.rs build rules) — The proc-macro expansion happens when the library is compiled. Bazel needs the dist folder present during this step.
- `rust_binary(compile_data = ["//frontend:dist"])` (in src/main.rs build rules) — The binary links the library, which already has embedded assets. This is redundant for the embedding itself, but ensures the dist folder is available if any binary-level code references it.

### Bazel Version Constraints

- **Minimum:** Bazel 7.5.0 (for `rules_rust` + `rules_js` compatibility)
- **Current:** Bazel 9.1.0 (for `aspect_rules_js 3.0.3` + `aspect_rules_ts 3.8.1`)

### TypeScript & Linting

- **TypeScript:** Pinned to exact version (5.5.4, not ~5.5) to ensure `aspect_rules_ts` reproducibility.
- **ESLint:** Flat config with neverthrow plugin registered. **Note (Phase 5 cycle-3 pivot):** `must-use-result` rule is disabled due to `@typescript-eslint` 8.x / flat-config incompatibility. The guardrail is enforced at code-review time instead.
- **Type Safety:** `@typescript-eslint/no-explicit-any: error` enforces explicit typing.

### Summary of Trade-offs

| Aspect | Gain | Cost |
|--------|------|------|
| **Frontend builds** | Simple, predictable | Not hermetic in Bazel |
| **Local workflow** | Fast feedback (Tilt watches src/) | Requires pnpm in dev env |
| **CI** | Explicit bootstrap step | Clear build requirements |
| **Binary** | Assets always embedded | Larger debug binaries |
| **Maintenance** | Less Bazel configuration | External build tool (pnpm) required |

This design prioritizes **clarity and reliability** over maximal hermeticity.
