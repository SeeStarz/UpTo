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
use serde::{Deserialize, Serialize};
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

    let server_socket = env::var("SERVER_ADDR")
        .map(|addr| {
            addr.strip_prefix("https://")
                .or(addr.strip_prefix("http://"))
                .unwrap_or(&addr)
                .to_string()
        })
        .unwrap_or(String::from("127.0.0.1:8000"));
    let listener = tokio::net::TcpListener::bind(&server_socket).await.unwrap();
    println!("Starting server on {}", server_socket);
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

#[derive(Clone, Debug, Serialize, Deserialize)]
struct APIFact {
    agent_id: String,
    age_seconds: u32,
    privacy: PrivacyStatus,
    #[serde(flatten)]
    fact: Fact,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GetAPIResponse {
    role: Role,
    staleness_ttl: u32,
    facts: Vec<APIFact>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
enum Role {
    Anonymous,
    User,
    Admin,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
enum PrivacyStatus {
    Public,
    Private,
}

#[axum::debug_handler]
async fn get_api(
    State(state): State<AppStateShared>,
    headers: HeaderMap,
) -> Result<Json<GetAPIResponse>, StatusCode> {
    let staleness_ttl = 60;
    let token = get_token(&headers);
    let role = if let Some(token) = token {
        let state = state.lock().await;
        if token == state.admin_token {
            Role::Admin
        } else if token == state.user_token {
            Role::User
        } else {
            eprintln!("Attempted invalid token: {}", token);
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        Role::Anonymous
    };
    let filtered = {
        use Role::*;
        match role {
            Admin | User => false,
            Anonymous => true,
        }
    };

    let Some(snapshot) = state.lock().await.snapshot.clone() else {
        return Err(StatusCode::NO_CONTENT);
    };

    let elapsed_time = Instant::now().duration_since(snapshot.last_updated);

    if elapsed_time > Duration::from_secs(staleness_ttl as u64) {
        return Err(StatusCode::NO_CONTENT);
    }

    let facts: Vec<APIFact> = snapshot
        .inner
        .facts
        .into_iter()
        .filter(|fact| {
            !filtered
                || fact
                    .metadata
                    .get("private")
                    .is_none_or(|value| value != "true")
        })
        .map(|fact| APIFact {
            privacy: if fact
                .metadata
                .get("private")
                .is_none_or(|value| value != "true")
            {
                PrivacyStatus::Public
            } else {
                PrivacyStatus::Private
            },
            agent_id: String::from("agent-laptop"),
            age_seconds: elapsed_time.as_secs() as u32,
            fact: fact,
        })
        .collect();
    Ok(Json(GetAPIResponse {
        role,
        staleness_ttl,
        facts,
    }))
}
