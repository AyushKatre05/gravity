-- Migration: Create complexities table
CREATE TABLE IF NOT EXISTS complexities (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    function_id     UUID NOT NULL REFERENCES functions(id) ON DELETE CASCADE,
    score           INTEGER NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_complexities_project_id  ON complexities(project_id);
CREATE INDEX IF NOT EXISTS idx_complexities_function_id ON complexities(function_id);
