# Safety rail: refuse to run outside the local kind context.
allow_k8s_contexts("kind-rust-app-template")

# Bazel builds the binary; the thin Dockerfile wraps it.
custom_build(
    ref = "rust-app-template:dev",
    command = """
        (cd frontend && pnpm install --frozen-lockfile >/dev/null 2>&1 && pnpm build >/dev/null 2>&1) && \\
        bazel build //:app && \\
        cp -f bazel-bin/app ./app && \\
        docker build -t $EXPECTED_REF .
    """,
    # IMPORTANT: only list source files here. Listing a build *output*
    # (e.g. `frontend/dist`) causes an infinite rebuild loop because the
    # build command regenerates it, which Tilt sees as a file change.
    deps = [
        "BUILD.bazel",
        "Cargo.toml",
        "Cargo.lock",
        "MODULE.bazel",
        "src",
        "Dockerfile",
        "frontend/src",
        "frontend/index.html",
        "frontend/vite.config.ts",
        "frontend/tsconfig.json",
        "frontend/package.json",
        "frontend/pnpm-lock.yaml",
    ],
)

k8s_yaml(kustomize("k8s/overlays/local"))

k8s_resource("local-rust-app-template", port_forwards = ["8080:8080"])
k8s_resource("local-postgres", port_forwards = ["5432:5432"])
