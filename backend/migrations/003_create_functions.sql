-- Migration: Create functions table
CREATE TABLE IF NOT EXISTS functions (
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id   UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    file_id      UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    line_start   INTEGER NOT NULL DEFAULT 0,
    line_end     INTEGER NOT NULL DEFAULT 0,
    is_public    BOOLEAN NOT NULL DEFAULT FALSE,
    is_async     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_functions_project_id ON functions(project_id);
CREATE INDEX IF NOT EXISTS idx_functions_file_id    ON functions(file_id);
