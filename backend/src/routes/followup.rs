use actix_web::{post, get, web, HttpResponse, Responder, HttpRequest};
use serde::Deserialize;
use async_stream::stream;
use std::time::Duration;

#[derive(Deserialize)]
struct FollowupRequest {
    session_id: String,
    question: String,
}

#[post("/api/followup")]
async fn followup(req: web::Json<FollowupRequest>) -> impl Responder {
    // For dev mode, return a stream_url pointing to /api/followup/{session_id}/stream
    let stream_url = format!("/api/followup/{}/stream", req.session_id);
    HttpResponse::Ok().json(serde_json::json!({"stream_url": stream_url}))
}

#[get("/api/followup/{id}/stream")]
async fn followup_stream(path: web::Path<String>, _req: HttpRequest) -> impl Responder {
    let id = path.into_inner();

    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "quack.db".to_string());
    let trace_id = uuid::Uuid::new_v4().to_string();

    let s = stream! {
        let content1 = format!("### **Follow-up Response (stub): {id}**\n\nThis is a simulated follow-up reply.\n");
        let db_path_clone = db_path.clone();
        let id_clone = id.clone();
        let c1 = content1.clone();
        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(conn) = crate::db::pool::get_connection(&db_path_clone) {
                let _ = crate::services::session::append_ai_response(&conn, &id_clone, &c1);
                let _ = crate::services::session::create_message(&conn, &id_clone, "assistant", &c1);
            }
        }).await;
        yield Ok::<_, actix_web::Error>(actix_web::web::Bytes::from(format!("event: chunk\ndata: {{\"content\":\"{}\", \"trace_id\": \"{}\"}}\n\n", content1, trace_id)));
        tokio::time::sleep(Duration::from_millis(200)).await;

        let content2 = "I recommend checking the types and ensuring conversions are correct.\n".to_string();
        let db_path_clone = db_path.clone();
        let id_clone = id.clone();
        let c2 = content2.clone();
        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(conn) = crate::db::pool::get_connection(&db_path_clone) {
                let _ = crate::services::session::append_ai_response(&conn, &id_clone, &c2);
                let _ = crate::services::session::create_message(&conn, &id_clone, "assistant", &c2);
            }
        }).await;
        yield Ok(actix_web::web::Bytes::from(format!("event: chunk\ndata: {{\"content\":\"{}\", \"trace_id\": \"{}\"}}\n\n", content2, trace_id)));
        tokio::time::sleep(Duration::from_millis(200)).await;

        let done = "event: done\ndata: {}\n\n".to_string();
        let db_path_clone = db_path.clone();
        let id_clone = id.clone();
        let content_done = "\n\n[followup stream done]\n".to_string();
        let cd = content_done.clone();
        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(conn) = crate::db::pool::get_connection(&db_path_clone) {
                let _ = crate::services::session::append_ai_response(&conn, &id_clone, &cd);
                let _ = crate::services::session::create_message(&conn, &id_clone, "assistant", &cd);
            }
        }).await;
        yield Ok(actix_web::web::Bytes::from(done));
    };

    HttpResponse::Ok()
        .insert_header((actix_web::http::header::CONTENT_TYPE, "text/event-stream"))
        .insert_header((actix_web::http::header::CACHE_CONTROL, "no-cache"))
        .streaming(s)
}
