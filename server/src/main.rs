use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use common::Snapshot;
use std::{println, sync::Arc};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

type AppState = Arc<Mutex<Option<Snapshot>>>;

#[tokio::main]
async fn main() {
    let state: AppState = Arc::new(Mutex::new(None));

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api", post(post_api).get(get_api))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn post_api(State(state): State<AppState>, Json(payload): Json<Snapshot>) {
    println!("{:?}", payload);

    let mut snapshot = state.lock().await;
    *snapshot = Some(payload);
}

async fn get_api(State(state): State<AppState>) -> Json<Option<Snapshot>> {
    let snapshot = state.lock().await;

    Json(snapshot.clone())
}
