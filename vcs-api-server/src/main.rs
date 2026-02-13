use axum::body::Bytes;
use axum::http::{HeaderMap, StatusCode};
use axum::{Json, Router, routing::get};
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let app = Router::new()
        .route("/", get(root))
        .route("/github", axum::routing::post(github_webhook));

    let listener = TcpListener::bind("0.0.0.0:8001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

fn verify_signature(secret: &[u8], body: &[u8], signature_header: &str) -> bool {
    let Some(hex_sig) = signature_header.strip_prefix("sha256=") else {
        return false;
    };

    let Ok(sig_bytes) = hex::decode(hex_sig) else {
        return false;
    };

    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret) else {
        return false;
    };

    mac.update(body);
    mac.verify_slice(&sig_bytes).is_ok()
}

async fn github_webhook(headers: HeaderMap, body: Bytes) -> Result<Json<Value>, StatusCode> {
    let secret =
        std::env::var("GITHUB_WEBHOOK_SECRET").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    if !verify_signature(secret.as_bytes(), &body, signature) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let payload: Value = serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;

    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    Ok(Json(payload))
}
