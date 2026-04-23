# frontend/

Vite + vanilla-TypeScript demo UI, built by Bazel into `dist/`, embedded into
the Rust binary via `rust-embed`.

## Deleting the frontend (headless consumers)

If your service does not need a browser UI:

1. Remove `frontend/` entirely: `rm -rf frontend/ pnpm-workspace.yaml`
2. Remove the JS rules from `MODULE.bazel`: delete the
   `aspect_rules_js`, `aspect_rules_ts`, `rules_nodejs`,
   `aspect_bazel_lib`, and `@npm` blocks. Keep `rules_rust`.
3. In root `BUILD.bazel`, delete the `compile_data = ["//frontend:dist"]`
   attribute on `rust_binary(name = "app", ...)`.
4. Delete `rust-embed` from `Cargo.toml` and rerun `just bazel-repin`.
5. In `src/http/mod.rs`, replace the `Assets` serving code with a stub:
   ```rust
   async fn root() -> &'static str { "rust-app-template (headless)" }
   // Router: .route("/", get(root))  — drop the /assets/*path route.
   ```
6. Remove the `bazel run //frontend:lint` line from `just check`.
7. `bazel build //:app` + `just test` should still pass.

Following these steps as a dry-read must resolve to files that exist in this
template — if any step references a file that isn't present, file an issue.
