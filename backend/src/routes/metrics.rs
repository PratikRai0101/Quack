use actix_web::{get, HttpResponse, Responder};

#[get("/api/metrics")]
async fn get_metrics() -> impl Responder {
    let snap = crate::services::metrics::snapshot();
    HttpResponse::Ok().json(snap)
}
