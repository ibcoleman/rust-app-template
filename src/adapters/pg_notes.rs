// @EXAMPLE-FILE notes
// Postgres adapter for the `Note` example domain — deleted by
// `just clean-examples`. Kept as a concrete reference of the
// `Port → sqlx → domain type` mapping for DB-backed adapters.

use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, PgPool};
use time::OffsetDateTime;

use crate::domain::{NoteId, MAX_NOTE_BODY_LEN};
use crate::ports::{NewNote, Note, NoteRepository, RepoError};

#[derive(Clone)]
pub struct PgNotes {
    pool: PgPool,
}

impl PgNotes {
    /// Connect to Postgres and run the bundled migrations.
    pub async fn connect(database_url: &str) -> Result<Self, RepoError> {
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect(database_url)
            .await
            .map_err(|e| RepoError::Backend(format!("connect: {e}")))?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| RepoError::Backend(format!("migrate: {e}")))?;

        Ok(Self { pool })
    }

    /// Clear all notes from the database. Only available with the `test-helpers` feature.
    #[cfg(feature = "test-helpers")]
    pub async fn reset_for_tests(&self) -> Result<(), RepoError> {
        sqlx::query("TRUNCATE notes")
            .execute(&self.pool)
            .await
            .map_err(|e| RepoError::Backend(format!("reset: {e}")))?;
        Ok(())
    }
}

#[async_trait]
impl NoteRepository for PgNotes {
    async fn create(&self, new: NewNote) -> Result<Note, RepoError> {
        if new.body.len() > MAX_NOTE_BODY_LEN {
            return Err(RepoError::Validation(format!(
                "body exceeds {MAX_NOTE_BODY_LEN} bytes (got {})",
                new.body.len()
            )));
        }

        let id = NoteId::new_v4();
        let created_at: OffsetDateTime = sqlx::query_scalar!(
            r#"INSERT INTO notes (id, body) VALUES ($1, $2) RETURNING created_at"#,
            id.as_uuid(),
            new.body,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepoError::Backend(format!("create: {e}")))?;

        Ok(Note {
            id,
            body: new.body,
            created_at,
        })
    }

    async fn get(&self, id: NoteId) -> Result<Option<Note>, RepoError> {
        let row = sqlx::query!(
            r#"SELECT id, body, created_at FROM notes WHERE id = $1"#,
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepoError::Backend(format!("get: {e}")))?;

        Ok(row.map(|r| Note {
            id: NoteId(r.id),
            body: r.body,
            created_at: r.created_at,
        }))
    }

    async fn list(&self, limit: u32) -> Result<Vec<Note>, RepoError> {
        let rows = sqlx::query!(
            r#"SELECT id, body, created_at FROM notes ORDER BY created_at DESC LIMIT $1"#,
            i64::from(limit)
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepoError::Backend(format!("list: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| Note {
                id: NoteId(r.id),
                body: r.body,
                created_at: r.created_at,
            })
            .collect())
    }
}
