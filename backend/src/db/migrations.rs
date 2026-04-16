use rusqlite::{params, Connection, Result};

pub fn run_migrations(db_path: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            command TEXT NOT NULL,
            working_dir TEXT,
            stdout TEXT NOT NULL DEFAULT '',
            stderr TEXT NOT NULL DEFAULT '',
            exit_code INTEGER NOT NULL DEFAULT -1,
            os_context TEXT NOT NULL DEFAULT '',
            git_context TEXT,
            project_type TEXT,
            ai_response TEXT NOT NULL DEFAULT '',
            model_used TEXT NOT NULL DEFAULT '',
            provider_used TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            duration_ms INTEGER
        );

        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(id),
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        "#,
    )?;

    Ok(())
}
