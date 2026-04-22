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
redis = { version = "0.27", features = ["tokio-comp"] }
```

Regenerate Bazel pins:

```bash
just bazel-repin
```

## 2. Create the decorator adapter

Create `src/adapters/cached_notes.rs`:

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
    async fn create(&self, new: NewNote) -> Result<Note, RepoError> { /* ... */ }
    async fn get(&self, id: NoteId) -> Result<Option<Note>, RepoError> { /* ... */ }
    async fn list(&self, limit: u32) -> Result<Vec<Note>, RepoError> { /* ... */ }
}
```

Re-export it from `src/adapters/mod.rs`:

```rust
pub mod cached_notes;
pub use cached_notes::CachedNotes;
```

## 3. Wire it into `AppState`

Edit `src/main.rs` — after constructing `PgNotes`, wrap it:

```rust
let pg = Arc::new(rust_app_template::adapters::PgNotes::connect(&database_url).await?);
let redis = redis::Client::open(std::env::var("REDIS_URL")?)?
    .get_connection_manager().await?;
let notes: Arc<dyn rust_app_template::ports::NoteRepository> =
    Arc::new(rust_app_template::adapters::CachedNotes::new(pg, redis));
```

## 4. Add a fake + test

Append to `tests/support/mod.rs`: an `InMemoryCache` fake.

Add a case to `tests/api.rs` that seeds a note, issues `GET /api/notes/:id`
twice, and asserts the second call served from cache (e.g., by counting
inner-repo calls on a test-only instrumented fake).

## 5. Add the Redis sidecar to k8s

Add `k8s/base/redis-deployment.yaml` + `k8s/base/redis-service.yaml`, and
register them in `k8s/base/kustomization.yaml`. The kustomize overlay at
`k8s/overlays/local/kustomization.yaml` will pick them up automatically.

## 6. Run `just test && just dev`

Verify the service still boots and the new cache path works end-to-end.
