use actix_web::{get, put, web, HttpResponse, Responder};
use crate::db::pool::get_connection;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct SettingsResp {
    api_key: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    base_url: Option<String>,
}

#[derive(Deserialize)]
struct SettingsReq {
    api_key: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    base_url: Option<String>,
}

fn mask_key(k: &str) -> String {
    if k.len() <= 8 {
        "****".to_string()
    } else {
        let start = &k[0..4];
        let end = &k[k.len()-4..];
        format!("{}****{}", start, end)
    }
}

#[get("/api/settings")]
async fn get_settings() -> impl Responder {
    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "quack.db".to_string());
    match get_connection(&db_path) {
        Ok(conn) => match crate::services::settings::list_settings(&conn) {
            Ok(map) => {
                let api_key = map.get("api_key").cloned().map(|k| mask_key(&k));
                let provider = map.get("provider").cloned();
                let model = map.get("model").cloned();
                let base_url = map.get("base_url").cloned();
                HttpResponse::Ok().json(SettingsResp { api_key, provider, model, base_url })
            }
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("DB error: {}", e)})),
        },
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("DB open failed: {}", e)})),
    }
}

#[put("/api/settings")]
async fn put_settings(req: web::Json<SettingsReq>) -> impl Responder {
    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "quack.db".to_string());
    match get_connection(&db_path) {
        Ok(conn) => {
            if let Some(api_key) = &req.api_key {
                if let Err(e) = crate::services::settings::set_setting(&conn, "api_key", api_key) {
                    return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("DB error: {}", e)}));
                }
            }
            if let Some(provider) = &req.provider {
                if let Err(e) = crate::services::settings::set_setting(&conn, "provider", provider) {
                    return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("DB error: {}", e)}));
                }
            }
            if let Some(model) = &req.model {
                if let Err(e) = crate::services::settings::set_setting(&conn, "model", model) {
                    return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("DB error: {}", e)}));
                }
            }
            if let Some(base_url) = &req.base_url {
                if let Err(e) = crate::services::settings::set_setting(&conn, "base_url", base_url) {
                    return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("DB error: {}", e)}));
                }
            }

            // Return masked settings
            match crate::services::settings::list_settings(&conn) {
                Ok(map) => {
                    let api_key = map.get("api_key").cloned().map(|k| mask_key(&k));
                    let provider = map.get("provider").cloned();
                    let model = map.get("model").cloned();
                    let base_url = map.get("base_url").cloned();
                    HttpResponse::Ok().json(SettingsResp { api_key, provider, model, base_url })
                }
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("DB error: {}", e)})),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("DB open failed: {}", e)})),
    }
}
