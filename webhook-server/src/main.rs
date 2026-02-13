use axum::{Json, Router, routing::get};
use serde_json::Value;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/github", axum::routing::post(webhook));

    let listener = TcpListener::bind("0.0.0.0:8001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn webhook(Json(payload): Json<Value>) -> Json<Value> {
    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    Json(payload)
}
