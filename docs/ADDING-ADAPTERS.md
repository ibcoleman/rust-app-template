# Adding Adapters

This template's hexagonal shape keeps business logic I/O-free: domain logic
depends on port traits (`src/ports/*.rs`), and concrete adapters
(`src/adapters/*.rs`) are injected via `AppState` in `src/main.rs`. To add a
new adapter, follow this worked example: a Redis cache decorator in front of
`NoteRepository`.

## 1. Add the crate dependency

Edit `Cargo.toml`:

```toml
[dependencies]
redis = { version = "0.27", features = ["tokio-comp", "connection-manager"] }
```

Both features are required: `tokio-comp` enables the async `aio` module on
tokio; `connection-manager` enables `aio::ConnectionManager` specifically.
Leaving off `connection-manager` gives a confusing "cannot find type
`ConnectionManager`" error even though `redis::aio` resolves. This
walkthrough was tested against `redis = "0.27"`; the 1.x API shifted and
would need further edits.

Regenerate Bazel pins:

```bash
just bazel-repin
```

The library targets `:lib` and `:lib_with_test_helpers` use
`all_crate_deps()`, so they pick up the new crate automatically — no
`BUILD.bazel` edit needed for them. The binary `:app` and `rust_test`
targets keep explicit deps lists; see step 3 below.

## 2. Create the decorator adapter

(You will create this file.)

Create `src/adapters/cached_notes.rs`. Start with pass-through bodies that
delegate to the inner repo — no caching yet. Step 4 layers the real caching
behavior on top.

```rust
use std::sync::Arc;
use async_trait::async_trait;
use redis::aio::ConnectionManager;

use crate::domain::NoteId;
use crate::ports::{NewNote, Note, NoteRepository, RepoError};

pub struct CachedNotes {
    inner: Arc<dyn NoteRepository>,
    redis: ConnectionManager,
}

impl CachedNotes {
    pub fn new(inner: Arc<dyn NoteRepository>, redis: ConnectionManager) -> Self {
        Self { inner, redis }
    }
}

#[async_trait]
impl NoteRepository for CachedNotes {
    async fn create(&self, new: NewNote) -> Result<Note, RepoError> {
        self.inner.create(new).await
    }
    async fn get(&self, id: NoteId) -> Result<Option<Note>, RepoError> {
        self.inner.get(id).await
    }
    async fn list(&self, limit: u32) -> Result<Vec<Note>, RepoError> {
        self.inner.list(limit).await
    }
}
```

Do **not** use `{ /* ... */ }` (the block returns `()`, which doesn't
match the signature — won't compile) or `todo!()` (type-checks but panics
at runtime — the moment step 3 wires this into `AppState`, every note
request panics in the worker thread, axum drops the connection, and
the browser shows `TypeError: Failed to fetch` with no visible error on
the server side except a stack trace in the pod logs).

Re-export it from `src/adapters/mod.rs`:

```rust
pub mod cached_notes;
pub use cached_notes::CachedNotes;
```

## 3. Wire it into `AppState`

Edit `src/main.rs` — after constructing `PgNotes`, wrap it:

```rust
let pg = Arc::new(superbox::adapters::PgNotes::connect(&database_url).await?);
let redis = redis::Client::open(std::env::var("REDIS_URL")?)?
    .get_connection_manager().await?;
let notes: Arc<dyn superbox::ports::NoteRepository> =
    Arc::new(superbox::adapters::CachedNotes::new(pg, redis));
```

Since `main.rs` now directly imports `redis`, add `"@crates//:redis"` to
the `:app` target's explicit `deps` list in `BUILD.bazel`. Forgetting this
produces `unresolved import` at Bazel link time (even though `cargo check`
would be happy).

Two startup failure modes to watch for:

- `std::env::var("REDIS_URL")?` returns `Err` if the variable isn't set,
  and `main` exits. Under Tilt this shows as the app container
  crashlooping with `Error: environment variable not found`. Step 5
  supplies the env var.
- `.get_connection_manager().await?` does a synchronous DNS/TCP resolve
  before returning. If the hostname in `REDIS_URL` doesn't resolve (wrong
  service name in the overlay — see step 5), the call hangs with no error
  log. You'll only see it as readiness-probe `connection refused` failures
  on the app pod, because the HTTP server never binds.

## 4. Add a fake + test

Testing the cache requires a swappable backend, which means a small
refactor first. Rough shape:

1. Add `src/ports/cache.rs` defining `trait CacheBackend` with `async get`
   / `async set` methods and a `CacheError` enum. Re-export from
   `src/ports/mod.rs`.
2. Change `CachedNotes` to hold `Arc<dyn CacheBackend>` instead of
   `ConnectionManager`. Move the Redis wiring into a `RedisCache` adapter
   at `src/adapters/redis_cache.rs` that implements the port.
3. Add two fakes to `tests/support/mod.rs`:
   - `InMemoryCache` — `CacheBackend` backed by `Mutex<HashMap<String, Vec<u8>>>`.
   - `CountingNotes<R: NoteRepository>` — decorator around any repo that
     bumps an `AtomicU64` on each `get()` call. Exposes `get_count()` for
     test assertions.
4. Add a case to `tests/api.rs` that wires
   `CachedNotes::new(CountingNotes::new(InMemoryNoteRepository::new()),
   InMemoryCache::default())` into `AppState`, seeds a note, issues `GET
   /api/notes/:id` twice, and asserts `counting.get_count() == 1`.

Keep a concrete `Arc<CountingNotes<_>>` handle alongside the one you pass
into `CachedNotes::new` — Rust's `CoerceUnsized` promotes it to
`Arc<dyn NoteRepository>` at the call site, but you need the concrete
handle afterward to read the counter.

## 5. Add the Redis sidecar to k8s

(You will create these files.)

Add `k8s/base/redis-deployment.yaml` (a cache doesn't need persistent
storage — a plain `Deployment` is fine, no `StatefulSet`/PVC) and
`k8s/base/redis-service.yaml` (`ClusterIP` on port 6379). Register both in
`k8s/base/kustomization.yaml`.

Then add `REDIS_URL` to the `env:` block in `k8s/base/deployment.yaml`:

```yaml
- name: REDIS_URL
  value: redis://redis:6379
```

**Overlay patch (easy to miss).** `k8s/overlays/local/kustomization.yaml`
applies a `local-` namePrefix to every resource, so the in-cluster service
is `local-redis` — not `redis`. There's already a patch rewriting
`DATABASE_URL` to use `local-postgres`; mirror that for `REDIS_URL`:

```yaml
- op: replace
  path: /spec/template/spec/containers/0/env/3/value
  value: redis://local-redis:6379
```

Index `3` assumes `REDIS_URL` is the 4th entry under `env:` — verify the
ordering in your base deployment. If you skip this patch, the pod starts,
`get_connection_manager()` hangs on an unresolvable hostname, and only
`connection refused` readiness failures show up; there's no error log
pointing at Redis.

## 6. Run `just test && just dev`

Verify the service still boots and the new cache path works end-to-end.
