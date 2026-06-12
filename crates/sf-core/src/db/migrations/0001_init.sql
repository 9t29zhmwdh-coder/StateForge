CREATE TABLE IF NOT EXISTS state_machines (
    id           TEXT PRIMARY KEY NOT NULL,
    name         TEXT NOT NULL,
    source_json  TEXT NOT NULL,
    states_json  TEXT NOT NULL DEFAULT '[]',
    transitions_json TEXT NOT NULL DEFAULT '[]',
    initial_state TEXT,
    context_type TEXT,
    tags_json    TEXT NOT NULL DEFAULT '[]',
    ai_summary   TEXT,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS projects (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL,
    description TEXT,
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS project_machines (
    project_id TEXT NOT NULL,
    machine_id TEXT NOT NULL,
    PRIMARY KEY (project_id, machine_id)
);

CREATE TABLE IF NOT EXISTS app_settings (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_machines_name       ON state_machines (name);
CREATE INDEX IF NOT EXISTS idx_machines_updated    ON state_machines (updated_at DESC);
