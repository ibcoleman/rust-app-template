# Roadmap

Tracked enhancements for this template. Ordered within each group by
bang-for-buck (how much friction the fix removes per line of change).

Last updated: 2026-04-23

## Completed

- Untrack built `/app` binary; ignore `.jj` state
- Pod + container `securityContext` on app Deployment and postgres StatefulSet
  (`runAsNonRoot`, pinned UID/GID, read-only root filesystem, dropped
  capabilities, seccomp `RuntimeDefault`). Added `emptyDir` mounts on
  `/var/run/postgresql` and `/tmp` so postgres still works under a read-only
  root.

## In flight

_(none)_

## DX: first-run / onboarding friction

- **Seed data** — `just dev` currently lands on an empty UI. Add `just seed`
  (or a Tilt resource that runs post-migration) that inserts a handful of
  notes so a fresh clone demos something real.
- **`.env.example`** — commit a template covering `DATABASE_URL`, `BIND_ADDR`,
  `RUST_LOG`. The README and CLAUDE.md both reference `DATABASE_URL` but
  nothing ships an example; `just test-integration` silently skips without it.
- **`just doctor` should check versions**, not just presence. `bazel 6` vs
  `bazel 7`, `kind 0.15` vs `0.20`, `pnpm` major mismatches are classic
  lost-afternoon material.

## DX: daily-loop speed

- **Cache frontend build in dev flows** — `pnpm install --frozen-lockfile &&
  pnpm build` runs inside `just dev`, `just test`, `just check`, and `just
  test-integration`. A staleness check (compare `frontend/dist` mtime to
  `frontend/src` + `frontend/package.json`) would skip the ~several-second
  rebuild when nothing changed.
- **Debug shortcuts** — add `just logs [component]`, `just psql`, `just
  shell [component]` (probably all thin wrappers around `kubectl exec` /
  `kubectl logs` on the running kind cluster). These are ~10 lines each and
  replace a "remember the kubectl incantation" step every single session.
- **`just fmt`** — currently only `just fix` (fmt + clippy fix). Plain `fmt`
  is common muscle memory.

## DX: template extensibility story

- **`just scaffold-adapter <name>`** — `docs/ADDING-ADAPTERS.md` is solid
  prose, but the template's flagship extensibility story should be one
  command that stubs port trait + adapter module + mod wiring + a test file.
  Prose-only puts the bus factor on whoever wrote the doc.
- **`just new-migration <name>`** — adding a migration today means "pick the
  next number, match the filename format, hope." Should be one command that
  emits `migrations/NNNN_name.sql` with the right number prefix.
- **`just add-dep <crate>` for Rust** — `just add-fe-dep` exists for the
  frontend; Rust side is a Cargo.toml edit plus `just bazel-repin` (and
  sometimes a manual BUILD.bazel edit per CLAUDE.md). A wrapper that handles
  the common case mirrors the frontend recipe.
- **Expose `scripts/bulk-rename.sh` as `just rename <new-name>`** —
  post-clone rename is the first thing a new user does; it should be a
  Just recipe, not a "read the docs to find the script" step.

## DX: feedback loops

- **Run integration tests on every PR** — currently gated behind
  `workflow_dispatch`, so "CI green" is misleading. Switch to a `services:
  postgres` block on the PR trigger.
- **Pre-commit hook / `just install-hooks`** — fmt/clippy failures get
  caught in CI instead of at commit time. `.pre-commit-config.yaml` or a
  minimal `.git/hooks/pre-commit` installer is a shift-left win.
- **Generate frontend types from Rust** — `frontend/src/types.ts` is
  hand-maintained and duplicates Rust domain types. Every backend schema
  change is a silent drift waiting to bite. Wire `utoipa` (or `schemars`)
  → OpenAPI / JSON Schema → `openapi-typescript` so the TS file is
  codegen'd.

## DX: polish

- **`.http` file or `docs/API-EXAMPLES.md`** — "how do I exercise the API
  while `just dev` is up?" currently means reading source. A checked-in
  REST Client `.http` file (or equivalent curl snippets) closes that gap.
- **`just reset-db`** — today the only reset is `just reset-cluster`
  (nuclear). A light-weight path that drops + re-migrates without
  rebuilding the kind cluster is a daily-use nicety.

## Production-hardening candidates (separate track)

These came out of an initial security pass but are deliberately split off
from the DX work above — they belong in a `k8s/overlays/prod/` layer, not
in `base/`, so the local dev loop stays friction-free.

- Resource requests / limits as a prod overlay patch
- `NetworkPolicy` (default-deny + explicit allows) in the prod overlay,
  with a comment that kindnet doesn't enforce NetworkPolicy — enforcement
  requires Calico/Cilium in the target cluster
- Pin Dockerfile base image by digest (`FROM debian:trixie-slim@sha256:...`)
  plus a Renovate/Dependabot note so digests don't rot through unpatched
  CVEs
- `docs/DEPLOYMENT.md` documenting the path from plaintext dev secrets to
  production Secrets (Kubernetes Secrets → SealedSecrets / ExternalSecrets
  / SOPS), image promotion flow, and rollout notes

## Observability / production readiness (separate track)

- Request-ID middleware + span context on errors
- `/metrics` endpoint via `metrics` or `prometheus` crate
- Split `/healthz` into `/startup` / `/ready` / `/live` so a DB outage
  fails readiness instead of killing the pod
- CORS, security headers, request timeout, body size limit
- Structured error codes on `ApiError` (replace `BadRequest(String)`)
- Typed `Config` struct instead of ad-hoc env reads
- Remote Bazel cache wired up (BuildBuddy setup documented in CLAUDE.md
  but not landed)
