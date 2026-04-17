use anyhow::Result;
use rusqlite::{params, Connection};
use std::collections::HashMap;

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;
    if let Some(row) = rows.next()? {
        let v: String = row.get(0)?;
        Ok(Some(v))
    } else {
        Ok(None)
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn list_settings(conn: &Connection) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let k: String = row.get(0)?;
        let v: String = row.get(1)?;
        map.insert(k, v);
    }
    Ok(map)
}
