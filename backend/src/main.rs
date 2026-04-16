use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use std::time::Duration;
use async_stream::stream;
use futures_core::Stream;
use actix_web::body::BodyStream;
use actix_web::http::header::{CONTENT_TYPE, CACHE_CONTROL};

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
    // For initial iteration we return a session stub and simulated stderr
    let session_id = Uuid::new_v4().to_string();
    let res = AnalyzeResponse {
        session_id: session_id.clone(),
        command: req.command.clone(),
        stdout: "".to_string(),
        stderr: "simulated error: file not found".to_string(),
        exit_code: 1,
        os_context: format!("OS: {}", std::env::consts::OS),
        has_git_context: false,
        project_type: None,
        created_at: Utc::now().to_rfc3339(),
    };

    HttpResponse::Ok().json(res)
}

#[get("/api/analyze/{id}/stream")]
async fn analyze_stream(path: web::Path<String>, _req: HttpRequest) -> impl Responder {
    // Stubbed SSE stream for frontend development. Emits a few chunks and a done event.
    let id = path.into_inner();

    let s = stream! {
        // First chunk: analysis header
        yield Ok::<_, actix_web::Error>(actix_web::web::Bytes::from(format!(
            "event: chunk\ndata: {{\"content\":\"### **Analysis: {id}**\\n\\nThis is a simulated analysis stream for frontend development.\\n\"}}\n\n"
        )));
        tokio::time::sleep(Duration::from_millis(250)).await;

        // Second chunk: the glitch
        yield Ok(actix_web::web::Bytes::from("event: chunk\ndata: {\"content\":\"### **The Glitch**\\nA simulated compiler error: mismatched types.\\n\"}\n\n"));
        tokio::time::sleep(Duration::from_millis(250)).await;

        // Third chunk: the solution (fenced code)
        yield Ok(actix_web::web::Bytes::from("event: chunk\ndata: {\"content\":\"### **The Solution**\\n```rust\\nlet x: i32 = 42;\\n```\\n\"}\n\n"));
        tokio::time::sleep(Duration::from_millis(250)).await;

        // Done event
        yield Ok(actix_web::web::Bytes::from("event: done\ndata: {}\n\n"));
    };

    let body = BodyStream::new(s);

    HttpResponse::Ok()
        .insert_header((CONTENT_TYPE, "text/event-stream"))
        .insert_header((CACHE_CONTROL, "no-cache"))
        .streaming(body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string()).parse().unwrap_or(3001);
    println!("Starting quack-server on http://127.0.0.1:{}", port);

    HttpServer::new(|| {
        App::new()
            .service(health)
            .service(analyze)
            .service(analyze_stream)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
