use axum::{
    routing::post,
    Json, Router, Server,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

use tauri::{AppHandle, Emitter};

#[derive(Deserialize)]
struct SetRequest {
    path: String
}

#[derive(Serialize)]
struct SetResponse {
    success: bool
}

async fn handle_set(app: AppHandle, Json(payload): Json<SetRequest>) -> Json<SetResponse> {
    println!("Received path: {}", payload.path);
    
    app.emit("update-wallpaper", payload.path.clone()).unwrap();

    Json(SetResponse {
        success: true
    })
}

pub async fn start_server(_app: AppHandle) {
    let app = Router::new()
        .route("/set", post(|payload| handle_set(_app,payload)))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("Server listening on {}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
