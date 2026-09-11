#![allow(dead_code)]

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use tokio::sync::{watch, RwLock};
use uuid::Uuid;

use crate::lumin_core::models::{
    AnalysisEvent, AnalysisEventAck, EndSession, SessionBroadcast, StudentInfo, StudentList,
};
use crate::network::auth::{generate_join_code, JoinCodeState};

pub struct ServerState {
    pub session_id: Option<String>,
    pub join_code: Arc<JoinCodeState>,
    pub teacher_token: String,
    pub active_session: RwLock<Option<Uuid>>,
    pub students: RwLock<Vec<StudentInfo>>,
    pub analysis_events: RwLock<Vec<AnalysisEvent>>,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// `GET /health` — returns 200 OK when the server is up. (Unauthenticated.)
async fn health() -> StatusCode {
    StatusCode::OK
}

/// `GET /session` — returns basic session metadata.
/// Protected by join-code middleware.
async fn session_info(State(state): State<Arc<ServerState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "session_id": state.session_id,
    }))
}

/// `POST /code` — generate a new random 4-digit join code.
/// Protected by `X-Lumin-Teacher-Token` header (teacher only).
async fn regenerate_code(
    State(state): State<Arc<ServerState>>,
    req: Request<Body>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Teacher-only guard: require the teacher token header.
    let token = req
        .headers()
        .get("X-Lumin-Teacher-Token")
        .and_then(|v| v.to_str().ok());
    if token != Some(state.teacher_token.as_str()) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let new_code = generate_join_code();
    *state.join_code.code.write().await = Some(new_code.clone());

    Ok(Json(serde_json::json!({
        "join_code": new_code,
    })))
}

/// `GET /code` — return the current join code.
/// Protected by `X-Lumin-Teacher-Token` header (teacher only).
async fn get_code(
    State(state): State<Arc<ServerState>>,
    req: Request<Body>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let token = req
        .headers()
        .get("X-Lumin-Teacher-Token")
        .and_then(|v| v.to_str().ok());
    if token != Some(state.teacher_token.as_str()) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let code = state.join_code.code.read().await.clone();
    Ok(Json(serde_json::json!({
        "join_code": code,
    })))
}

/// `POST /session` — broadcast session + quiz to students.
/// Teacher-only: requires `X-Lumin-Teacher-Token` header.
async fn broadcast_session(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<SessionBroadcast>,
) -> Result<StatusCode, StatusCode> {
    let mut session = state.active_session.write().await;
    *session = Some(payload.session_id);
    Ok(StatusCode::OK)
}

/// `POST /session/end` — teacher ends the session.
/// Teacher-only: requires `X-Lumin-Teacher-Token` header.
async fn end_session(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<EndSession>,
) -> Result<StatusCode, StatusCode> {
    let mut session = state.active_session.write().await;
    if *session == Some(payload.session_id) {
        *session = None;
        let mut students = state.students.write().await;
        students.clear();
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

/// `POST /analysis` — student submits an analysis event.
/// Returns event_id in response body.
async fn submit_analysis(
    State(state): State<Arc<ServerState>>,
    Json(event): Json<AnalysisEvent>,
) -> Result<Json<AnalysisEventAck>, StatusCode> {
    let mut events = state.analysis_events.write().await;
    events.push(event.clone());
    Ok(Json(AnalysisEventAck {
        event_id: event.id,
        received_at: Utc::now(),
    }))
}

/// `POST /analysis/ack` — teacher acknowledges an event.
/// Teacher-only: requires `X-Lumin-Teacher-Token` header.
async fn ack_analysis(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<AnalysisEventAck>,
) -> Result<StatusCode, StatusCode> {
    let mut events = state.analysis_events.write().await;
    events.retain(|e| e.id != payload.event_id);
    Ok(StatusCode::OK)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct StudentJoinBody {
    participant_token: Option<String>,
}

/// `POST /students/join` — register a student for the active session.
///
/// 参加コード(X-Lumin-Join-Code)での認証のみを要求する(セッション照合はコード自体が担う)。
async fn students_join(
    State(state): State<Arc<ServerState>>,
    axum::Json(body): axum::Json<StudentJoinBody>,
) -> Json<serde_json::Value> {
    let token = body
        .participant_token
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let mut students = state.students.write().await;
    if !students.iter().any(|s| s.participant_token == token) {
        students.push(StudentInfo {
            participant_token: token.clone(),
            connected_at: Utc::now(),
        });
    }
    Json(serde_json::json!({ "participantToken": token }))
}

/// `GET /students` — list connected students.
/// Teacher-only: requires `X-Lumin-Teacher-Token` header.
async fn list_students(State(state): State<Arc<ServerState>>) -> Json<StudentList> {
    let students = state.students.read().await;
    Json(StudentList {
        students: students.clone(),
    })
}

// ── Middleware (teacher-token guard for management endpoints) ─────────────────

/// Middleware that validates the `X-Lumin-Teacher-Token` header on
/// teacher-only management routes.
async fn teacher_token_middleware(
    State(state): State<Arc<ServerState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = req
        .headers()
        .get("X-Lumin-Teacher-Token")
        .and_then(|v| v.to_str().ok());
    if token == Some(state.teacher_token.as_str()) {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn session_uuid_middleware(
    State(state): State<Arc<ServerState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let header_session_id = req
        .headers()
        .get("X-Lumin-Session-ID")
        .and_then(|v| v.to_str().ok());

    let active_session = state.active_session.read().await;

    match (header_session_id, active_session.as_ref()) {
        (Some(id), Some(expected)) => {
            if id == expected.to_string() {
                Ok(next.run(req).await)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
        _ => Err(StatusCode::FORBIDDEN),
    }
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Start the HTTP server on all interfaces (`0.0.0.0`).
///
/// The server shuts down gracefully when `shutdown_rx` is updated.
///
/// Returns `(server_handle, teacher_token)` where `teacher_token` is an opaque
/// string the local UI can use to call teacher-only endpoints.
pub async fn start_server(
    port: u16,
    session_id: String,
    initial_join_code: String,
    shutdown_rx: watch::Receiver<bool>,
) -> Result<(tokio::task::JoinHandle<()>, String), String> {
    let teacher_token = uuid::Uuid::new_v4().to_string();

    let state = Arc::new(ServerState {
        session_id: Some(session_id),
        join_code: Arc::new(JoinCodeState::new(initial_join_code)),
        teacher_token: teacher_token.clone(),
        active_session: RwLock::new(None),
        students: RwLock::new(Vec::new()),
        analysis_events: RwLock::new(Vec::new()),
    });

    let app = build_router(state);

    let handle = start_server_with_router(port, app, shutdown_rx).await?;

    Ok((handle, teacher_token))
}

/// Build the full axum router from a shared [`ServerState`].
pub fn build_router(state: Arc<ServerState>) -> Router {
    // ── Public (unauthenticated) routes ──────────────────────────────────
    let public_routes = Router::new().route("/health", get(health));

    // ── Teacher-only management routes (teacher token required) ───────────
    let teacher_routes = Router::new()
        .route("/code", post(regenerate_code).get(get_code))
        .route("/session", post(broadcast_session).delete(end_session))
        .route("/analysis/ack", post(ack_analysis))
        .route("/students", get(list_students))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            teacher_token_middleware,
        ));

    // ── Classroom routes (join code + session UUID required) ──────────────
    let classroom_routes = Router::new()
        .route("/session", get(session_info))
        .route("/analysis", post(submit_analysis))
        .route("/students/join", post(students_join))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            session_uuid_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.join_code.clone(),
            crate::network::auth::join_code_middleware,
        ));

    public_routes
        .merge(teacher_routes)
        .merge(classroom_routes)
        .with_state(state)
}

/// Start serving the provided router on all interfaces (`0.0.0.0`).
///
/// Used by [`start_server`] and by callers that already own a [`ServerState`].
pub async fn start_server_with_router(
    port: u16,
    app: Router,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<tokio::task::JoinHandle<()>, String> {
    let addr: SocketAddr = ([0, 0, 0, 0], port).into();

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("failed to bind {addr}: {e}"))?;

    println!("[server] listening on {addr}");

    let handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.changed().await;
                println!("[server] shutting down");
            })
            .await
            .map_err(|e| eprintln!("[server] error: {e}"))
            .ok();
    });

    Ok(handle)
}

/// Start the HTTP server with a caller-provided shared state.
///
/// Useful when the teacher UI needs direct access to the same [`ServerState`]
/// the HTTP handlers use (e.g. to list or kick students without a teacher token).
pub async fn start_server_with_state(
    port: u16,
    state: Arc<ServerState>,
    shutdown_rx: watch::Receiver<bool>,
) -> Result<tokio::task::JoinHandle<()>, String> {
    let app = build_router(state);
    start_server_with_router(port, app, shutdown_rx).await
}

/// Discover the active LAN interface IP (best-effort).
pub fn discover_lan_ip() -> Option<String> {
    local_ip_address::local_ip().ok().map(|ip| ip.to_string())
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn test_state() -> Arc<ServerState> {
        Arc::new(ServerState {
            session_id: Some("test-session".into()),
            join_code: Arc::new(JoinCodeState::new("1234".into())),
            teacher_token: "test-token".into(),
            active_session: RwLock::new(Some(Uuid::new_v4())),
            students: RwLock::new(Vec::new()),
            analysis_events: RwLock::new(Vec::new()),
        })
    }

    #[test]
    fn test_discover_lan_ip() {
        let ip = super::discover_lan_ip();
        if let Some(ip) = ip {
            assert!(ip.contains('.'), "Expected IPv4");
        }
    }

    #[test]
    fn test_generate_join_code_length() {
        let code = generate_join_code();
        assert_eq!(code.len(), 4, "code must be exactly 4 characters: {code}");
        assert!(
            code.chars().all(|c| c.is_ascii_digit()),
            "code must be all digits: {code}"
        );
    }

    #[test]
    fn test_generate_join_code_range() {
        for _ in 0..100 {
            let code = generate_join_code().parse::<u32>().unwrap();
            assert!(code <= 9999, "code {code} exceeds maximum 9999");
        }
    }

    #[tokio::test]
    async fn students_join_registers_student_and_is_visible_to_teacher() {
        use crate::network::auth::JoinCodeState;

        let session_uuid = Uuid::new_v4();
        let state = Arc::new(ServerState {
            session_id: Some("s".into()),
            join_code: Arc::new(JoinCodeState::new("1234".into())),
            teacher_token: "teacher-token".into(),
            active_session: RwLock::new(Some(session_uuid)),
            students: RwLock::new(Vec::new()),
            analysis_events: RwLock::new(Vec::new()),
        });
        let router = build_router(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("http://127.0.0.1:{port}/students/join"))
            .header("X-Lumin-Join-Code", "1234")
            .header("X-Lumin-Session-ID", session_uuid.to_string())
            .json(&serde_json::json!({}))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "join must succeed with valid headers");
        let body: serde_json::Value = resp.json().await.unwrap();
        let token = body["participantToken"].as_str().unwrap().to_string();

        let list = client
            .get(format!("http://127.0.0.1:{port}/students"))
            .header("X-Lumin-Teacher-Token", "teacher-token")
            .send()
            .await
            .unwrap();
        assert!(list.status().is_success());
        let v: serde_json::Value = list.json().await.unwrap();
        let arr = v["students"].as_array().expect("students array");
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["participantToken"].as_str(), Some(token.as_str()));

        // 誤コードは401
        let bad = client
            .post(format!("http://127.0.0.1:{port}/students/join"))
            .header("X-Lumin-Join-Code", "9999")
            .header("X-Lumin-Session-ID", session_uuid.to_string())
            .json(&serde_json::json!({}))
            .send()
            .await
            .unwrap();
        assert_eq!(bad.status(), reqwest::StatusCode::UNAUTHORIZED);
        assert_eq!(state.students.read().await.len(), 1);
    }

    #[tokio::test]
    async fn test_join_code_state_lifecycle() {
        let state = JoinCodeState::default();
        assert!(state.code.read().await.is_none());

        *state.code.write().await = Some("1234".into());
        assert_eq!(*state.code.read().await.as_ref().unwrap(), "1234");

        let new_code = generate_join_code();
        *state.code.write().await = Some(new_code.clone());
        assert_eq!(*state.code.read().await.as_ref().unwrap(), new_code);
    }

    #[tokio::test]
    async fn test_submit_analysis_without_session_id_returns_403() {
        let state = test_state();
        let app = Router::new()
            .route("/analysis", post(submit_analysis))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                session_uuid_middleware,
            ))
            .with_state(state);

        let event = AnalysisEvent {
            id: Uuid::new_v4(),
            participant_token: "tok_abc".into(),
            session_id: Some(Uuid::new_v4()),
            question_id: "q1".into(),
            concept: "fractions".into(),
            misconception: None,
            correct: true,
            hint_count: 0,
            retry_success: false,
            submitted_at: Utc::now(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/analysis")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&event).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_submit_analysis_with_wrong_session_returns_403() {
        let state = test_state();
        let app = Router::new()
            .route("/analysis", post(submit_analysis))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                session_uuid_middleware,
            ))
            .with_state(state);

        let event = AnalysisEvent {
            id: Uuid::new_v4(),
            participant_token: "tok_abc".into(),
            session_id: Some(Uuid::new_v4()),
            question_id: "q1".into(),
            concept: "fractions".into(),
            misconception: None,
            correct: true,
            hint_count: 0,
            retry_success: false,
            submitted_at: Utc::now(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/analysis")
                    .header("content-type", "application/json")
                    .header("X-Lumin-Session-ID", "wrong-session-id")
                    .body(Body::from(serde_json::to_string(&event).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_submit_analysis_with_valid_session_returns_200() {
        let state = test_state();
        let session_id = state.active_session.read().await.unwrap();

        let app = Router::new()
            .route("/analysis", post(submit_analysis))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                session_uuid_middleware,
            ))
            .with_state(state);

        let event = AnalysisEvent {
            id: Uuid::new_v4(),
            participant_token: "tok_abc".into(),
            session_id: Some(session_id),
            question_id: "q1".into(),
            concept: "fractions".into(),
            misconception: None,
            correct: true,
            hint_count: 0,
            retry_success: false,
            submitted_at: Utc::now(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/analysis")
                    .header("content-type", "application/json")
                    .header("X-Lumin-Session-ID", session_id.to_string())
                    .body(Body::from(serde_json::to_string(&event).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_broadcast_session_updates_server_state() {
        let state = test_state();
        let new_session_id = Uuid::new_v4();

        let app = Router::new()
            .route("/session", post(broadcast_session))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                teacher_token_middleware,
            ))
            .with_state(state.clone());

        let payload = SessionBroadcast {
            session_id: new_session_id,
            quiz: crate::lumin_core::models::Quiz {
                id: "quiz1".into(),
                title: "Test Quiz".into(),
                subject: "Math".into(),
                topic: None,
                questions: vec![],
            },
            join_code: "1234".into(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/session")
                    .header("content-type", "application/json")
                    .header("X-Lumin-Teacher-Token", "test-token")
                    .body(Body::from(serde_json::to_string(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let active_session = state.active_session.read().await;
        assert_eq!(*active_session, Some(new_session_id));
    }
}
