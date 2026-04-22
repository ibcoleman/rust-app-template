# Architecture

This template follows **hexagonal architecture** (ports & adapters). Business logic is domain-pure in the center, with I/O pushed to the boundaries.

## Hexagonal Shape

```
                      ┌─────────────────┐
                      │   HTTP Adapter  │
                      │  (Axum router)  │
                      └────────┬────────┘
                               │
                    ┌──────────┴──────────┐
                    │                     │
         ┌──────────▼──────────┐  ┌──────▼──────────┐
         │  Domain Services    │  │  AppState DI    │
         │  (business logic)   │  │  (Arc<dyn>)     │
         └──────────┬──────────┘  └─────────────────┘
                    │
         ┌──────────▼──────────┐
         │   Ports (traits)    │
         │ NoteRepository, etc │
         └──────────┬──────────┘
                    │
        ┌───────────┴───────────┐
        │                       │
    ┌───▼──────┐       ┌───────▼────┐
    │  PgNotes │       │ MemoryNotes │
    │(Postgres)│       │   (tests)   │
    └──────────┘       └─────────────┘
```

The outer ring is infrastructure (adapters); the inner ring is business logic (domain + ports).

## Where Each Concern Lives

### `src/domain/`

**Value types with no I/O.** These are pure data structures and invariants:

- `NoteId(Uuid)` — newtype-wrapped, prevents `NoteId(user_id)` bugs
- `Note` — complete note record
- `NewNote` — creation request (validated)
- Domain constants: `MAX_NOTE_BODY_LEN`
- Domain rules: note IDs are unique, bodies have max length

Nothing in `domain/` imports `sqlx`, `redis`, or async code. Domain logic is deterministic and testable without infrastructure.

### `src/ports/`

**Async trait abstractions.** These are contracts between layers:

- `NoteRepository` — "I can create, read, list notes"
- `GreetingPort` — "I can generate a greeting"

Each port defines:
- Inputs (e.g., `NewNote`)
- Outputs (e.g., `Result<Note, RepoError>`)
- Async or sync
- Error types (domain-specific: `RepoError`, not `sqlx::Error`)

Ports are **not** implementations. They're boundaries. Services depend on port traits, never directly on concrete adapters.

### `src/adapters/`

**Concrete implementations of ports.** I/O happens here:

- `PgNotes` — PostgreSQL implementation of `NoteRepository`
  - Executes SQL queries via `sqlx`
  - Maps database rows to domain `Note` types
  - Converts SQL errors to `RepoError`
- `StaticGreeter` — hardcoded implementation of `GreetingPort`

Adapters know about infrastructure (databases, APIs, file systems). Domain logic doesn't.

### `src/http/`

**HTTP boundary.** Axum router, request handlers, response formatting:

- `mod.rs` — Axum router definition, shared state, embedded frontend assets handler
- `greet.rs` — handler for `GET /api/greet`
- `notes.rs` — handlers for `GET/POST /api/notes`, `GET /api/notes/:id`
- `error.rs` — `ApiError` enum, converts domain errors to HTTP status codes

The HTTP adapter is the outermost ring. It knows about domain + ports but not other adapters. Frontend assets are compiled into the binary via `rust-embed Assets` struct in `mod.rs` — not a separate file.

## Error Flow

Domain errors bubble outward and transform:

```
PgNotes::get(id)
  ↓
Returns Result<Option<Note>, RepoError>
  ↓
Handler catches RepoError
  ↓
Maps RepoError → ApiError (via impl From<RepoError> for ApiError)
  ↓
ApiError serializes to JSON + HTTP status
```

This separation means:

1. **Domain doesn't depend on HTTP.** You can reuse domain + ports in CLI, webhooks, gRPC.
2. **Infrastructure errors are translated.** Database errors (400 vs 500) become HTTP errors intelligently.
3. **Errors are explicit.** `thiserror` enums force exhaustiveness; can't accidentally lose error context.

## AppState + Dependency Injection

In `src/main.rs`, we construct `AppState`:

```rust
pub struct AppState {
    notes: Arc<dyn NoteRepository>,
    greeting: Arc<dyn GreetingPort>,
}
```

We inject concrete adapters:

```rust
let pg = Arc::new(PgNotes::connect(&database_url).await?);
let greeting = Arc::new(StaticGreeter);

let state = AppState {
    notes: pg,
    greeting,
};
```

**Why `Arc<dyn Trait>`?**

- `Arc` — thread-safe shared ownership, required for Axum extractors across requests
- `dyn Trait` — runtime polymorphism. Handlers don't know if `notes` is `PgNotes` or `MemoryNotes`
- Enables decoration (wrap one adapter in another, e.g., caching layer)

**When to wrap vs. nest:**

- **Wrap (decorate):** Add a cross-cutting concern without changing the inner adapter. Example: `CachedNotes` wraps `PgNotes`, adds Redis cache, forwards cache misses to `PgNotes`.
- **Nest (DI):** One adapter depends on another. Example: `PgNotes` depends on a database connection pool (injected).

## Build Shape

Bazel is the single engine:

- **`rules_rust`** — builds Rust crates from Cargo.toml
- **`crate_universe`** — auto-generates BUILD files from Cargo.toml (never hand-written)
- **`rules_js`** — builds frontend Vite app via pnpm
- **Final binary** — multi-stage Docker build, embeds frontend assets via `rust-embed`

One Bazel graph:

```
//... (top-level)
├── //:lib (Rust core + domain + adapters + ports)
├── //:bin (HTTP server)
└── //frontend (TypeScript)
    ↓ (built)
    └── assets.tar
        ↓ (embedded)
        └── //:bin (final binary)
```

`cargo fmt`, `cargo clippy` remain on Cargo (toolchain maturity). Everything else flows through Bazel.

## Testing Seams

### Unit + Property Tests (`tests/properties.rs`, `tests/api.rs`)

Offline, deterministic. They use fakes from `tests/support/mod.rs`:

```rust
struct MemoryNotes { notes: Arc<Mutex<HashMap<...>>> }

#[async_trait]
impl NoteRepository for MemoryNotes {
    // In-memory, no database
}
```

Property tests verify invariants hold for all inputs:

```rust
proptest! {
    #[test]
    fn note_id_never_zero(id in uuid::any()) {
        let note_id = NoteId(id);
        assert_ne!(note_id.as_uuid(), Uuid::nil());
    }
}
```

### Integration Tests (`tests/integration_db.rs`)

Real PostgreSQL via Testcontainers (or docker-compose in CI). Marked `#[ignore]` + Bazel `manual` tag:

```bash
bazel test //tests:integration_db --config=live
```

These verify the real adapter (e.g., `PgNotes`) against a schema without mocks.

## Dependency Inversion

```
┌─ Ports (owned by domain)
│
├─ Domain (no I/O, depends only on domain + ports)
│
├─ HTTP Adapter (knows about domain + ports, depends on both)
│
└─ Other Adapters (PgNotes, etc., implement ports, know about infrastructure)
```

**Direction:** Arrows point inward. Domain is at the center and has no outbound dependencies.

**Benefits:**
1. Domain is testable without infrastructure.
2. Adapters are swappable (real DB ↔ mock).
3. New adapters (Redis, different DB) don't affect domain or HTTP.

## Key Files

- `src/domain/mod.rs` — domain types
- `src/ports/mod.rs` — trait definitions
- `src/adapters/mod.rs` — concrete implementations
- `src/http/mod.rs` — router and error mapping
- `src/main.rs` — AppState wiring
- `tests/support/mod.rs` — fakes
- `tests/properties.rs` — property tests
- `tests/api.rs` — integration tests (API level)
