use rusqlite::{Connection, Result};
use std::path::Path;

/// Simple helper to open or create the SQLite database at `db_path`.
/// For MVP we use a single-connection model; later we can replace with a pool.
/// This helper also configures WAL mode and a busy timeout to improve
/// concurrency when multiple threads write to the DB (e.g., streaming tasks).
pub fn get_connection(db_path: &str) -> Result<Connection> {
    // Ensure parent directory exists
    if let Some(parent) = Path::new(db_path).parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let conn = Connection::open(db_path)?;
    // Configure pragmatic defaults for durability and concurrency
    // Enable WAL for better concurrent reads/writes and set a busy timeout.
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL; PRAGMA busy_timeout = 5000;")?;
    Ok(conn)
}
