pub const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS profiles (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    mode_type   TEXT NOT NULL,
    triggers    TEXT NOT NULL DEFAULT '[]',
    actions     TEXT NOT NULL DEFAULT '[]',
    restore_policy TEXT NOT NULL DEFAULT 'on_app_quit',
    approval_mode  TEXT NOT NULL DEFAULT 'ask_first',
    is_builtin  INTEGER NOT NULL DEFAULT 0,
    enabled     INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id              TEXT PRIMARY KEY,
    profile_id      TEXT NOT NULL REFERENCES profiles(id),
    trigger_source  TEXT NOT NULL,
    started_at      TEXT NOT NULL,
    ended_at        TEXT,
    status          TEXT NOT NULL DEFAULT 'active',
    snapshot        TEXT,
    safe_mode       INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS activity_logs (
    id              TEXT PRIMARY KEY,
    session_id      TEXT,
    profile_id      TEXT,
    event_type      TEXT NOT NULL,
    action_kind     TEXT,
    trigger_kind    TEXT,
    result          TEXT NOT NULL DEFAULT 'ok',
    failure_reason  TEXT,
    restore_status  TEXT,
    metadata        TEXT,
    created_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS approved_apps (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    process_name TEXT NOT NULL,
    action      TEXT NOT NULL DEFAULT 'pause',
    profile_ids TEXT NOT NULL DEFAULT '[]',
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS approved_scripts (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    script_path TEXT NOT NULL,
    phase       TEXT NOT NULL DEFAULT 'start',
    timeout_secs INTEGER NOT NULL DEFAULT 30,
    profile_id  TEXT REFERENCES profiles(id),
    approved    INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key     TEXT PRIMARY KEY,
    value   TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_activity_logs_session ON activity_logs(session_id);
CREATE INDEX IF NOT EXISTS idx_activity_logs_created ON activity_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
";
