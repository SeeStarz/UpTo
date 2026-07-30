use axum::{
    Json, Router,
    extract::State,
    http::{
        HeaderMap, StatusCode,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    routing::{get, post},
};
use common::{Fact, Snapshot as SnapshotCommon};
use std::{env, eprintln, println, sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::Instant};
use tower_http::cors::CorsLayer;

type AppStateShared = Arc<Mutex<AppState>>;

#[derive(Clone, Debug)]
struct Snapshot {
    inner: SnapshotCommon,
    last_updated: Instant,
}

#[derive(Clone, Debug)]
struct AppState {
    snapshot: Option<Snapshot>,
    admin_token: String,
    user_token: String,
}

impl AppState {
    fn new() -> Self {
        let snapshot = None;
        let admin_token = env::var("ADMIN_TOKEN").unwrap_or(String::from("admin_btw"));
        let user_token = env::var("USER_TOKEN").unwrap_or(String::from("just_user"));

        AppState {
            snapshot,
            admin_token,
            user_token,
        }
    }
}

#[tokio::main]
async fn main() {
    let state: AppStateShared = Arc::new(Mutex::new(AppState::new()));
    {
        let unlocked = state.lock().await;
        println!(
            "Token setup:\nUSER_TOKEN: {}\nADMIN_TOKEN: {}",
            unlocked.user_token, unlocked.admin_token,
        );
    }

    println!("Server starting!");

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api", post(post_api).get(get_api))
        .layer(CorsLayer::permissive().allow_headers([AUTHORIZATION, CONTENT_TYPE]))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn get_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|header_value| header_value.to_str().ok())
        .and_then(|header_value| header_value.trim().strip_prefix("Custom "))
        .map(|header_value| header_value.to_string())
}

#[axum::debug_handler]
async fn post_api(
    State(state): State<AppStateShared>,
    headers: HeaderMap,
    Json(payload): Json<SnapshotCommon>,
) -> Result<(), StatusCode> {
    let Some(token) = get_token(&headers) else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if token != state.lock().await.admin_token {
        eprintln!("Attempted invalid token: {}", token);
        return Err(StatusCode::FORBIDDEN);
    }

    println!("{:?}", payload);
    let mut unlocked = state.lock().await;
    unlocked.snapshot = Some(Snapshot {
        inner: payload,
        last_updated: Instant::now(),
    });
    Ok(())
}

#[axum::debug_handler]
async fn get_api(
    State(state): State<AppStateShared>,
    headers: HeaderMap,
) -> Result<Json<SnapshotCommon>, StatusCode> {
    let token = get_token(&headers);

    let Some(snapshot) = state.lock().await.snapshot.clone() else {
        return Err(StatusCode::NO_CONTENT);
    };

    if Instant::now().duration_since(snapshot.last_updated) > Duration::from_secs(60) {
        return Err(StatusCode::NO_CONTENT);
    }

    if let Some(token) = token {
        if token == state.lock().await.admin_token || token == state.lock().await.user_token {
            return Ok(Json(snapshot.inner));
        } else {
            eprintln!("Attempted invalid token: {}", token);
        }
    }

    let filtered: Vec<Fact> = snapshot
        .inner
        .facts
        .into_iter()
        .filter(|fact| {
            fact.metadata
                .get("private")
                .is_none_or(|value| value != "true")
        })
        .collect();
    let snapshot = SnapshotCommon { facts: filtered };
    Ok(Json(snapshot))
}
