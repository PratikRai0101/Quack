use rusqlite::{Connection, Result};
use std::path::Path;

/// Simple helper to open or create the SQLite database at `db_path`.
/// For MVP we use a single-connection model; later we can replace with a pool.
pub fn get_connection(db_path: &str) -> Result<Connection> {
    // Ensure parent directory exists
    if let Some(parent) = Path::new(db_path).parent() {
        std::fs::create_dir_all(parent).ok();
    }
    Connection::open(db_path)
}
