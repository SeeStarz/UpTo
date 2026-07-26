use axum::{
    Json, Router,
    routing::{get, post},
};
use common::Snapshot;
use std::println;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api", post(post_api));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    println!("Hello");
    axum::serve(listener, app).await.unwrap();
}

async fn post_api(Json(payload): Json<Snapshot>) {
    eprintln!("{:?}", payload);
}
