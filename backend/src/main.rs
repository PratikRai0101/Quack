use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::time::Duration;
use async_stream::stream;
use futures_util::StreamExt;
use actix_web::http::header::{CONTENT_TYPE, CACHE_CONTROL};

use tracing::{info, error};
use tracing_subscriber;

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
    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "quack.db".to_string());
    let llm_cfg = crate::services::llm::LlmConfig::from_db_or_env(&db_path);
    let provider = llm_cfg.provider.clone();
    let provider_configured = llm_cfg.api_key.is_some();
    HttpResponse::Ok().json(serde_json::json!({"status":"ok","version":"2.0.0","provider": provider, "provider_configured": provider_configured}))
}

#[post("/api/analyze")]
async fn analyze(req: web::Json<AnalyzeRequest>) -> impl Responder {
    // Run the command via shell service
    tracing::info!(command = %req.command, "analyze.request");
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
                            // increment analyze count metric
                            crate::services::metrics::incr_analyze();
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

    // Decide whether to use real LLM or stubbed streaming
    let llm_cfg = crate::services::llm::LlmConfig::from_db_or_env(&db_path);
    let use_stub = std::env::var("QUACK_STUB_LLM").unwrap_or_else(|_| "1".to_string()) == "1";

    let s = stream! {
        // generate a trace id for this stream
        let trace_id = uuid::Uuid::new_v4().to_string();
        if !use_stub && llm_cfg.provider == "groq" {
            // Try to stream real LLM output for this session
            match crate::db::pool::get_connection(&db_path) {
                Ok(conn) => {
                    match crate::services::session::get_session(&conn, &id) {
                        Ok(Some(sess)) => {
                            // Call into LLM service with actual session data
                            let mut llm_stream = Box::pin(crate::services::llm::stream_analysis(&llm_cfg, &sess.command, &sess.stdout, &sess.stderr, sess.git_context.clone(), &sess.os_context));
                            // log start
                                                    tracing::info!(session_id = %id, trace_id = %trace_id, "llm.stream.session.start");
                            crate::services::metrics::incr_llm_stream_start();
                            while let Some(item) = llm_stream.as_mut().next().await {
                                match item {
                                    Ok(chunk) => {
                                        // persist chunk and message
                                        let db_path_clone = db_path.clone();
                                        let id_clone = id.clone();
                                        let chunk_clone = chunk.clone();
                                        let _ = tokio::task::spawn_blocking(move || {
                                            if let Ok(conn2) = crate::db::pool::get_connection(&db_path_clone) {
                                                let _ = crate::services::session::append_ai_response(&conn2, &id_clone, &chunk_clone);
                                                let _ = crate::services::session::create_message(&conn2, &id_clone, "assistant", &chunk_clone);
                                            }
                                        }).await;

                                        // send SSE chunk with JSON payload including trace_id
                                        let payload = serde_json::json!({"content": chunk, "trace_id": trace_id});
                                        let data = format!("event: chunk\ndata: {}\n\n", payload.to_string());
                                        yield Ok::<_, actix_web::Error>(actix_web::web::Bytes::from(data));
                                    }
                                    Err(e) => {
                                        let payload = serde_json::json!({"message": format!("LLM error: {}", e), "trace_id": trace_id});
                                        let data = format!("event: error\ndata: {}\n\n", payload.to_string());
                                        let _ = yield Ok::<_, actix_web::Error>(actix_web::web::Bytes::from(data));
                                        break;
                                    }
                                }
                            }
                            // done
                            let done = "event: done\ndata: {}\n\n".to_string();
                            let _ = tokio::task::spawn_blocking({
                                let db_path_clone = db_path.clone();
                                let id_clone = id.clone();
                                let trace_clone = trace_id.clone();
                                move || {
                                    if let Ok(conn2) = crate::db::pool::get_connection(&db_path_clone) {
                                        let _ = crate::services::session::append_ai_response(&conn2, &id_clone, &format!("\n\n[stream done] trace_id:{}\n", trace_clone));
                                        let _ = crate::services::session::create_message(&conn2, &id_clone, "assistant", &format!("\n\n[stream done] trace_id:{}\n", trace_clone));
                                    }
                                }
                            }).await;
                            yield Ok(actix_web::web::Bytes::from(done));
                        }
                        Ok(None) => {
                            let msg = serde_json::json!({"error": "session not found", "trace_id": trace_id});
                            yield Ok(actix_web::web::Bytes::from(format!("event: error\ndata: {}\n\n", msg.to_string())));
                        }
                        Err(e) => {
                            let msg = serde_json::json!({"error": format!("DB error: {}", e), "trace_id": trace_id});
                            yield Ok(actix_web::web::Bytes::from(format!("event: error\ndata: {}\n\n", msg.to_string())));
                        }
                    }
                }
                Err(e) => {
                    let msg = serde_json::json!({"error": format!("DB open failed: {}", e), "trace_id": trace_id});
                    yield Ok(actix_web::web::Bytes::from(format!("event: error\ndata: {}\n\n", msg.to_string())));
                }
            }
        } else {
            // Fallback to stubbed analysis
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
            let chunk2 = "event: chunk\ndata: {\"content\":\"### **The Glitch**\\nA simulated compiler error: mismatched types.\\n\"}\\n\\n".to_string();
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
            let chunk3 = "event: chunk\ndata: {\"content\":\"### **The Solution**\\n```rust\\nlet x: i32 = 42;\\n```\\n\"}\\n\\n".to_string();
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
        }
    };

    HttpResponse::Ok()
        .insert_header((CONTENT_TYPE, "text/event-stream"))
        .insert_header((CACHE_CONTROL, "no-cache"))
        .streaming(s)
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing subscriber from environment (RUST_LOG)
    tracing_subscriber::fmt::init();

    // Load .env if present
    let _ = dotenvy::dotenv();

    // Initialize DB migrations
    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "quack.db".to_string());
    match run_migrations(&db_path) {
        Ok(_) => info!("Database migrations applied (db={})", db_path),
        Err(e) => error!("Failed to apply migrations: {}", e),
    }

    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string()).parse().unwrap_or(3001);
    info!("Starting quack-server on http://127.0.0.1:{}", port);

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
            .service(crate::routes::settings::get_settings)
            .service(crate::routes::settings::put_settings)
            .service(crate::routes::metrics::get_metrics)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
