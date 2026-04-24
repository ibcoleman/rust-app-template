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
- `just rename <kebab-name>` wrapping `scripts/bulk-rename.sh` (snake_case
  derived automatically).
- `just logs [component]`, `just psql`, `just shell [component]` — thin
  kubectl wrappers. Running `logs` or `shell` without args lists the
  available components from the current cluster.
- `just add-dep <crate> [args]` wrapping `cargo add` + `just bazel-repin`,
  with a reminder about the manual BUILD.bazel step for `:app` / `rust_test`
  targets.
- Cordoned the example domain with `@EXAMPLE-FILE` / `@EXAMPLE-BLOCK-START`
  / `@EXAMPLE-BLOCK-END` banners, plus `just clean-examples` to strip them.
  `Note` (DB-backed CRUD) is what gets removed; `Greeter` stays as a
  minimal reference of the port/adapter/handler pattern without a DB.
  Post-strip `cargo fmt` runs to normalize whitespace; verified `cargo
  check`, `clippy -D warnings`, `cargo fmt --check`, and `cargo test` all
  clean afterwards.

## In flight

_(none)_

## DX: first-run / onboarding friction

- **Seed data** — `just dev` currently lands on an empty UI. Add `just seed`
  (or a Tilt resource that runs post-migration) that inserts a handful of
  notes so a fresh clone demos something real.
- **`just doctor` should check versions**, not just presence. `bazel 6` vs
  `bazel 7`, `kind 0.15` vs `0.20`, `pnpm` major mismatches are classic
  lost-afternoon material.
- **`docs/STARTING-A-PROJECT.md`** — procedural guide for an agent or human
  kicking off a new project from this template: what order to make decisions
  in, what to delete, what to scaffold, what to defer. Today the template
  tells you *how to build* (hexagonal, `Result`, newtypes, property tests)
  but not *what order to build in* or *what decisions to surface early*.
  Highest-leverage meta-doc.
- **`docs/DECISIONS/`** — ADR-style stubs for common bolt-ons the template
  deliberately leaves open: auth/session, background jobs, rate-limiting,
  message queues, admin UI framework, email, file storage. Each doc: problem
  statement, 2–3 recommended options with tradeoffs, and "if you pick X, run
  `just add-X`." Prevents every clone reinventing the same decisions
  differently.
- **CLAUDE.md "bootstrapping a new project" section** — short pointer to
  `STARTING-A-PROJECT.md` plus the order rules (rename first, strip
  examples, scaffold domains, defer auth/workers until the domain is
  stable). CLAUDE.md is where agents actually look.
- **`.agents/prompts/`** — pre-written prompts for common template-onboarding
  operations (`new-domain`, `add-migration`, `add-auth`, etc.) that
  enumerate conventions and invoke the scaffolders. Lets agents skip
  re-deriving the house style every session.

## DX: daily-loop speed

- **Cache frontend build in dev flows** — `pnpm install --frozen-lockfile &&
  pnpm build` runs inside `just dev`, `just test`, `just check`, and `just
  test-integration`. A staleness check (compare `frontend/dist` mtime to
  `frontend/src` + `frontend/package.json`) would skip the ~several-second
  rebuild when nothing changed.

## DX: template extensibility story

- **`just scaffold-adapter <name>`** — `docs/ADDING-ADAPTERS.md` is solid
  prose, but the template's flagship extensibility story should be one
  command that stubs port trait + adapter module + mod wiring + a test file.
  Prose-only puts the bus factor on whoever wrote the doc.
- **`just new-domain <name>`** — full vertical-slice scaffolder (superset of
  `scaffold-adapter`): creates `src/domain/<name>/mod.rs` with `<Name>Id` +
  `<Name>Error` skeleton, port trait, postgres adapter, migration stub,
  property test skeleton, plus all module wiring. Complementary to
  `scaffold-adapter` (which is "add another adapter to an existing port");
  `new-domain` is "create a whole new entity."
- **`just new-migration <name>`** — adding a migration today means "pick the
  next number, match the filename format, hope." Should be one command that
  emits `migrations/NNNN_name.sql` with the right number prefix.
- **Reference outbound HTTP client adapter** at `src/adapters/http_client/`
  — `reqwest` with retries, timeouts, user-agent, structured tracing.
  Scrapers, API integrations, and webhook senders all rebuild this; a
  reference short-circuits the decision and anchors the pattern.
- **Richer reference migration** — `migrations/0001_notes.sql` is trivial
  (one table, no FKs). Add a second multi-entity example demonstrating
  house style: `TIMESTAMPTZ`, `uuid DEFAULT gen_random_uuid()`, foreign
  keys with `ON DELETE` clauses, composite unique constraints, index
  naming. Gives agents something concrete to copy.
- **`tests/fixtures/` convention + loader** — standard place for saved HTML
  / JSON payloads (fixture-based scraper tests, recorded API responses),
  plus a small helper that loads them by name. Referenced in the testing
  philosophy but no scaffolding today.
- **Second-binary scaffold** — decision + scaffolding for "this repo
  produces both an API server and a worker." Template is API-shaped today
  (single `src/main.rs` → HTTP server); adding a worker silently implies a
  Cargo.toml / BUILD.bazel / k8s Deployment restructure the agent has to
  design. Either bake in a `src/bin/worker.rs` stub with matching targets,
  or ship `just add-worker <name>`.
- **`just add-auth`** (controversial — needs stack decision first) —
  opinionated opt-in: drops in the recommended auth stack (tentatively
  axum-login + tower-sessions + argon2), adds `User` entity, wires
  login/logout/me endpoints, adds auth middleware. Every clone reinvents
  auth; a canonical scaffold saves weeks per-project.
- **`just add-admin-shell`** (controversial — needs stack decision first)
  — opinionated opt-in: minimal admin UI shell on the frontend (router,
  layout, login page, generic list/detail scaffold). Admin UIs are
  near-universal; starting from vanilla TS every time is wasteful.
- **`just add-api-client <name>`** — opt-in scaffolder that emits an
  outbound HTTP client module following the reference adapter pattern
  above. Common enough that one-command generation pays off.

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
- **CLAUDE.md invariant checklist** — an explicit "before commit, verify"
  list the agent runs through: all fallible ops return `Result`? all IDs
  newtyped? all new domains have a property test? migration follows house
  style? no `unwrap()` / `expect()` in production code? Gives agents a
  concrete quality gate to self-check against, catching convention drift
  before CI does.

## DX: polish

- **`.http` file or `docs/API-EXAMPLES.md`** — "how do I exercise the API
  while `just dev` is up?" currently means reading source. A checked-in
  REST Client `.http` file (or equivalent curl snippets) closes that gap.

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
