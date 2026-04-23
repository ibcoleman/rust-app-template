CREATE TABLE notes (
    id         UUID PRIMARY KEY,
    body       TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX notes_created_at_desc ON notes (created_at DESC);
