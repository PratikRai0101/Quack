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

    let s = stream! {
        yield Ok::<_, actix_web::Error>(actix_web::web::Bytes::from(format!(
            "event: chunk\ndata: {{\"content\":\"### **Follow-up Response (stub): {id}**\\n\\nThis is a simulated follow-up reply.\\n\"}}\n\n"
        )));
        tokio::time::sleep(Duration::from_millis(200)).await;

        yield Ok(actix_web::web::Bytes::from("event: chunk\ndata: {\"content\":\"I recommend checking the types and ensuring conversions are correct.\\n\"}\n\n"));
        tokio::time::sleep(Duration::from_millis(200)).await;

        yield Ok(actix_web::web::Bytes::from("event: done\ndata: {}\n\n"));
    };

    HttpResponse::Ok()
        .insert_header((actix_web::http::header::CONTENT_TYPE, "text/event-stream"))
        .insert_header((actix_web::http::header::CACHE_CONTROL, "no-cache"))
        .streaming(s)
}
