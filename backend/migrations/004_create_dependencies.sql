-- Migration: Create dependencies table
CREATE TABLE IF NOT EXISTS dependencies (
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id   UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    file_id      UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    source       TEXT NOT NULL,
    target       TEXT NOT NULL,
    kind         TEXT NOT NULL DEFAULT 'use',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dependencies_project_id ON dependencies(project_id);
CREATE INDEX IF NOT EXISTS idx_dependencies_source     ON dependencies(source);
