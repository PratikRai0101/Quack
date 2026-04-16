use actix_web::{get, delete, web, HttpResponse, Responder};
use crate::db::pool::get_connection;
use serde::Serialize;

#[derive(Serialize)]
struct SessionSummary {
    id: String,
    command: String,
    exit_code: i32,
    created_at: String,
    duration_ms: Option<i64>,
}

#[get("/api/history")]
async fn list_history() -> impl Responder {
    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "quack.db".to_string());
    match get_connection(&db_path) {
        Ok(conn) => {
            let mut stmt = match conn.prepare("SELECT id, command, exit_code, created_at, duration_ms FROM sessions ORDER BY created_at DESC") {
                Ok(s) => s,
                Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("DB prepare failed: {}", e)})),
            };
            let rows = stmt.query_map([], |row| {
                Ok(SessionSummary {
                    id: row.get(0)?,
                    command: row.get(1)?,
                    exit_code: row.get(2)?,
                    created_at: row.get(3)?,
                    duration_ms: row.get(4).ok(),
                })
            });

            match rows {
                Ok(iter) => {
                    let mut results = Vec::new();
                    for r in iter {
                        match r {
                            Ok(s) => results.push(s),
                            Err(e) => eprintln!("Row map error: {}", e),
                        }
                    }
                    HttpResponse::Ok().json(results)
                }
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("DB query failed: {}", e)})),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("DB open failed: {}", e)})),
    }
}

#[get("/api/history/{id}")]
async fn get_history(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "quack.db".to_string());
    match get_connection(&db_path) {
        Ok(conn) => match crate::services::session::get_session(&conn, &id) {
            Ok(Some(sess)) => HttpResponse::Ok().json(serde_json::json!({
                "id": sess.id,
                "command": sess.command,
                "stdout": sess.stdout,
                "stderr": sess.stderr,
                "exit_code": sess.exit_code,
                "os_context": sess.os_context,
                "git_context": sess.git_context,
                "project_type": sess.project_type,
                "ai_response": sess.ai_response,
                "model_used": sess.model_used,
                "provider_used": sess.provider_used,
                "created_at": sess.created_at,
            })),
            Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error":"session not found"})),
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("DB error: {}", e)})),
        },
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("DB open failed: {}", e)})),
    }
}

#[delete("/api/history/{id}")]
async fn delete_history(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "quack.db".to_string());
    match get_connection(&db_path) {
        Ok(conn) => match conn.execute("DELETE FROM sessions WHERE id = ?1", [id.as_str()]) {
            Ok(_) => HttpResponse::NoContent().finish(),
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("DB delete failed: {}", e)})),
        },
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("DB open failed: {}", e)})),
    }
}
