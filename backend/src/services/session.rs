use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

pub struct Session {
    pub id: String,
    pub command: String,
    pub working_dir: Option<String>,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub os_context: String,
    pub git_context: Option<String>,
    pub project_type: Option<String>,
    pub ai_response: String,
    pub model_used: String,
    pub provider_used: String,
    pub created_at: String,
}

pub fn create_session(conn: &Connection, command: &str, stdout: &str, stderr: &str, exit_code: i32, os_context: &str, git_context: Option<&str>, project_type: Option<&str>) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO sessions (id, command, working_dir, stdout, stderr, exit_code, os_context, git_context, project_type, ai_response, model_used, provider_used, created_at) VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8, '', '', '', ?9)",
        params![id, command, stdout, stderr, exit_code, os_context, git_context, project_type, now],
    )?;

    Ok(id)
}

pub fn get_session(conn: &Connection, id: &str) -> Result<Option<Session>> {
    let mut stmt = conn.prepare("SELECT id, command, working_dir, stdout, stderr, exit_code, os_context, git_context, project_type, ai_response, model_used, provider_used, created_at FROM sessions WHERE id = ?1")?;
    let mut rows = stmt.query(params![id])?;

    if let Some(row) = rows.next()? {
        let s = Session {
            id: row.get(0)?,
            command: row.get(1)?,
            working_dir: row.get(2)?,
            stdout: row.get(3)?,
            stderr: row.get(4)?,
            exit_code: row.get(5)?,
            os_context: row.get(6)?,
            git_context: row.get(7)?,
            project_type: row.get(8)?,
            ai_response: row.get(9)?,
            model_used: row.get(10)?,
            provider_used: row.get(11)?,
            created_at: row.get(12)?,
        };
        Ok(Some(s))
    } else {
        Ok(None)
    }
}

pub fn append_ai_response(conn: &Connection, session_id: &str, delta: &str) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET ai_response = ai_response || ?1 WHERE id = ?2",
        params![delta, session_id],
    )?;
    Ok(())
}

pub fn create_message(conn: &Connection, session_id: &str, role: &str, content: &str) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, session_id, role, content, now],
    )?;
    Ok(id)
}
