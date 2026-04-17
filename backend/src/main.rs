use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use std::time::Duration;
use async_stream::stream;
use futures_core::Stream;
use actix_web::http::header::{CONTENT_TYPE, CACHE_CONTROL};

mod db;
mod services;
mod routes;
use db::migrations::run_migrations;
use db::pool::get_connection;


#[derive(Deserialize)]
struct AnalyzeRequest {
    command: String,
    working_directory: Option<String>,
}

#[derive(Serialize)]
struct AnalyzeResponse {
    session_id: String,
    command: String,
    stdout: String,
    stderr: String,
    exit_code: i32,
    os_context: String,
    has_git_context: bool,
    project_type: Option<String>,
    created_at: String,
}

#[get("/api/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status":"ok","version":"2.0.0"}))
}

#[post("/api/analyze")]
async fn analyze(req: web::Json<AnalyzeRequest>) -> impl Responder {
    // Run the command via shell service
    match services::shell::replay_command(&req.command) {
        Ok(out) => {
            // Try to persist session
            let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "quack.db".to_string());
            match get_connection(&db_path) {
                Ok(conn) => {
                    let git_ctx = services::context::get_git_diff();
                    let os_ctx = services::context::detect_os();
                    let project_type = services::context::detect_project_type(None);
                    match services::session::create_session(&conn, &req.command, &out.stdout, &out.stderr, out.exit_code, &os_ctx, git_ctx.as_deref(), project_type.as_deref()) {
                        Ok(session_id) => {
                            let res = AnalyzeResponse {
                                session_id: session_id.clone(),
                                command: req.command.clone(),
                                stdout: out.stdout.clone(),
                                stderr: out.stderr.clone(),
                                exit_code: out.exit_code,
                                os_context: os_ctx,
                                has_git_context: git_ctx.is_some(),
                                project_type: project_type.clone(),
                                created_at: Utc::now().to_rfc3339(),
                            };
                            HttpResponse::Ok().json(res)
                        }
                        Err(e) => {
                            eprintln!("Failed to create session: {}", e);
                            HttpResponse::InternalServerError().json(serde_json::json!({"error":"Failed to persist session"}))
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to open DB: {}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({"error":"DB unavailable"}))
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to replay command: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error":"Failed to replay command"}))
        }
    }
}

#[get("/api/analyze/{id}/stream")]
async fn analyze_stream(path: web::Path<String>, _req: HttpRequest) -> impl Responder {
    // Stubbed SSE stream for frontend development. Emits a few chunks and a done event.
    let id = path.into_inner();

    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "quack.db".to_string());

    let s = stream! {
        // First chunk: analysis header
        let chunk1 = format!("event: chunk\ndata: {{\"content\":\"### **Analysis: {id}**\\n\\nThis is a simulated analysis stream for frontend development.\\n\"}}\n\n");
        // persist chunk when possible
        let db_path_clone = db_path.clone();
        let id_clone = id.clone();
        let content1 = "### **Analysis:".to_string() + &id_clone + "**\n\nThis is a simulated analysis stream for frontend development.\n";
        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(conn) = crate::db::pool::get_connection(&db_path_clone) {
                let _ = crate::services::session::append_ai_response(&conn, &id_clone, &content1);
                let _ = crate::services::session::create_message(&conn, &id_clone, "assistant", &content1);
            }
        }).await;
        yield Ok::<_, actix_web::Error>(actix_web::web::Bytes::from(chunk1));
        tokio::time::sleep(Duration::from_millis(250)).await;

        // Second chunk: the glitch
        let chunk2 = "event: chunk\ndata: {\"content\":\"### **The Glitch**\\nA simulated compiler error: mismatched types.\\n\"}\n\n".to_string();
        let db_path_clone = db_path.clone();
        let id_clone = id.clone();
        let content2 = "### **The Glitch**\nA simulated compiler error: mismatched types.\n".to_string();
        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(conn) = crate::db::pool::get_connection(&db_path_clone) {
                let _ = crate::services::session::append_ai_response(&conn, &id_clone, &content2);
                let _ = crate::services::session::create_message(&conn, &id_clone, "assistant", &content2);
            }
        }).await;
        yield Ok(actix_web::web::Bytes::from(chunk2));
        tokio::time::sleep(Duration::from_millis(250)).await;

        // Third chunk: the solution (fenced code)
        let chunk3 = "event: chunk\ndata: {\"content\":\"### **The Solution**\\n```rust\\nlet x: i32 = 42;\\n```\\n\"}\n\n".to_string();
        let db_path_clone = db_path.clone();
        let id_clone = id.clone();
        let content3 = "### **The Solution**\n```rust\nlet x: i32 = 42;\n```\n".to_string();
        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(conn) = crate::db::pool::get_connection(&db_path_clone) {
                let _ = crate::services::session::append_ai_response(&conn, &id_clone, &content3);
                let _ = crate::services::session::create_message(&conn, &id_clone, "assistant", &content3);
            }
        }).await;
        yield Ok(actix_web::web::Bytes::from(chunk3));
        tokio::time::sleep(Duration::from_millis(250)).await;

        // Done event
        let done = "event: done\ndata: {}\n\n".to_string();
        let db_path_clone = db_path.clone();
        let id_clone = id.clone();
        let content_done = "\n\n[stream done]\n".to_string();
        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(conn) = crate::db::pool::get_connection(&db_path_clone) {
                let _ = crate::services::session::append_ai_response(&conn, &id_clone, &content_done);
                let _ = crate::services::session::create_message(&conn, &id_clone, "assistant", &content_done);
            }
        }).await;
        yield Ok(actix_web::web::Bytes::from(done));
    };

    HttpResponse::Ok()
        .insert_header((CONTENT_TYPE, "text/event-stream"))
        .insert_header((CACHE_CONTROL, "no-cache"))
        .streaming(s)
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load .env if present
    let _ = dotenvy::dotenv();

    // Initialize DB migrations
    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "quack.db".to_string());
    match run_migrations(&db_path) {
        Ok(_) => println!("Database migrations applied (db={})", db_path),
        Err(e) => eprintln!("Failed to apply migrations: {}", e),
    }

    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string()).parse().unwrap_or(3001);
    println!("Starting quack-server on http://127.0.0.1:{}", port);

    HttpServer::new(|| {
        App::new()
            .service(health)
            .service(analyze)
            .service(analyze_stream)
            .service(crate::routes::history::list_history)
            .service(crate::routes::history::get_history)
            .service(crate::routes::history::delete_history)
            .service(crate::routes::followup::followup)
            .service(crate::routes::followup::followup_stream)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
