use std::sync::Arc;

use axum::http::StatusCode;

use tauri::{AppHandle, State};
use tokio::sync::{watch, Mutex};
use uuid::Uuid;

use crate::inference::session::AppState;
use crate::lumin_core::models::{AnalysisEvent, Quiz, StudentInfo};
use crate::lumin_core::LessonPlan;
use crate::network::auth::{generate_join_code, JoinCodeState};
use crate::network::dns_sd::{advertise_teacher, stop_advertise};
use crate::network::server::{start_server_with_state, ServerState};

const SESSION_PORT: u16 = 8765;

/// 教室(mDNSインスタンス)名。参加コードは秘匿情報のため名前に含めない。
/// 一意性確保のためセッションUUID先頭6桁(大文字hex)を添える。
fn classroom_instance_name(session_id: &Uuid) -> String {
    let hex = session_id.simple().to_string();
    format!("Lumin教室 {}", hex[..6].to_uppercase())
}

fn validate_student_join_payload(body: &mut serde_json::Value) -> Result<String, String> {
    let obj = body
        .as_object_mut()
        .ok_or_else(|| "参加情報が不正です".to_string())?;
    let token = obj
        .get("participantToken")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "参加情報が不正です".to_string())?
        .to_owned();
    if obj.get("quiz").is_none_or(|q| q.is_null()) {
        return Err("小テストが配信されていません".into());
    }
    Ok(token)
}

#[derive(Clone, serde::Serialize)]
pub struct StudentSummary {
    pub id: String,
    pub joined_at: String,
}

impl From<&StudentInfo> for StudentSummary {
    fn from(student: &StudentInfo) -> Self {
        Self {
            id: student.participant_token.clone(),
            joined_at: student.connected_at.to_rfc3339(),
        }
    }
}

pub struct SessionManager {
    server_state: Option<Arc<ServerState>>,
    server_handle: Option<tokio::task::JoinHandle<()>>,
    server_shutdown: Option<watch::Sender<bool>>,
    advertise_handle: Option<u64>,
    active_quiz: Option<Quiz>,
    session_id: Option<Uuid>,
    join_code: Option<String>,
    last_adopted_plan: Option<LessonPlan>,
    student_connection: Option<StudentConnection>,
}

#[derive(Clone)]
struct StudentConnection {
    base: String,
    session_id: Uuid,
    join_code: String,
    participant_token: String,
}

impl SessionManager {
    pub async fn send_student_analysis(&self, mut event: AnalysisEvent) -> Result<(), String> {
        let connection = self
            .student_connection
            .as_ref()
            .ok_or("教室に参加し直してください")?;
        event.session_id = Some(connection.session_id);
        event.participant_token = connection.participant_token.clone();
        reqwest::Client::new()
            .post(format!("{}/analysis", connection.base))
            .timeout(std::time::Duration::from_secs(10))
            .header("X-Lumin-Join-Code", &connection.join_code)
            .header("X-Lumin-Session-ID", connection.session_id.to_string())
            .json(&event)
            .send()
            .await
            .map_err(|e| format!("結果を送信できませんでした: {e}"))?
            .error_for_status()
            .map_err(|e| format!("結果を送信できませんでした: {e}"))?;
        Ok(())
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            server_state: None,
            server_handle: None,
            server_shutdown: None,
            advertise_handle: None,
            active_quiz: None,
            session_id: None,
            join_code: None,
            last_adopted_plan: None,
            student_connection: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.session_id.is_some()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeacherSessionSnapshot {
    quiz: Option<Quiz>,
    join_code: Option<String>,
}

#[tauri::command]
pub async fn get_teacher_session(
    state: State<'_, Mutex<SessionManager>>,
) -> Result<TeacherSessionSnapshot, String> {
    let manager = state.lock().await;
    Ok(TeacherSessionSnapshot {
        quiz: manager.active_quiz.clone(),
        join_code: manager.join_code.clone(),
    })
}

#[tauri::command]
pub async fn list_quizzes() -> Result<Vec<Quiz>, String> {
    use crate::lumin_core::demo_data;
    Ok(demo_data::demo_quiz_bank())
}

/// 学生が教室へ参加する: `http://{host}:{port}/students/join` を叩き、
/// 参加コード認証の下で自分をサーバの学生一覧へ登録する。
#[tauri::command]
pub async fn student_join(
    host: String,
    port: u16,
    session_id: Option<String>,
    join_code: String,
    _manager: tauri::State<'_, Mutex<SessionManager>>,
) -> Result<serde_json::Value, String> {
    if join_code.len() != 4 || !join_code.chars().all(|c| c.is_ascii_digit()) {
        return Err("参加コードは4桁の数字で入力してください".into());
    }
    if host.is_empty() || port == 0 {
        return Err("接続先が正しくありません".into());
    }
    let base = crate::network::dns_sd::classroom_base_url(&host, port);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("HTTPクライアント初期化に失敗しました: {e}"))?;
    let session_id = match session_id.filter(|s| !s.is_empty()) {
        Some(id) => id,
        None => {
            let response = client
                .get(format!("{base}/session/resolve"))
                .header("X-Lumin-Join-Code", &join_code)
                .send()
                .await
                .map_err(|e| format!("教師端末へ接続できませんでした: {e}"))?;
            if response.status() == StatusCode::UNAUTHORIZED {
                return Err("参加コードが正しくありません".into());
            }
            let body: serde_json::Value = response
                .error_for_status()
                .map_err(|e| e.to_string())?
                .json()
                .await
                .map_err(|e| e.to_string())?;
            body["sessionId"]
                .as_str()
                .ok_or("教室の情報が取得できませんでした")?
                .to_owned()
        }
    };
    let req = client
        .post(format!("{base}/students/join"))
        .header("X-Lumin-Join-Code", &join_code)
        .header("X-Lumin-Session-ID", &session_id);

    let resp = req
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| format!("教師端末へ接続できませんでした: {e}"))?;

    match resp.status() {
        StatusCode::OK => {
            let mut body: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("応答の解析に失敗: {e}"))?;
            let token = validate_student_join_payload(&mut body)?;
            let session_id = Uuid::parse_str(&session_id).map_err(|e| e.to_string())?;
            let obj = body
                .as_object_mut()
                .ok_or_else(|| "参加情報が不正です".to_string())?;
            obj.insert("host".into(), serde_json::Value::String(host));
            obj.insert("port".into(), serde_json::Value::from(port));
            _manager.lock().await.student_connection = Some(StudentConnection {
                base,
                session_id,
                join_code,
                participant_token: token,
            });
            Ok(body)
        }
        StatusCode::UNAUTHORIZED => Err("参加コードが正しくありません".into()),
        status => Err(format!("サーバがエラーを返しました ({status})")),
    }
}

#[tauri::command]
pub async fn start_session(
    app: AppHandle,
    quiz_id: String,
    inference: State<'_, AppState>,
    state: State<'_, Mutex<SessionManager>>,
) -> Result<String, String> {
    let mut manager = state.lock().await;

    if manager.is_active() {
        return Err("すでにセッションが進行中です".into());
    }

    let quizzes: Vec<Quiz> = {
        use crate::lumin_core::demo_data;
        demo_data::demo_quiz_bank()
    };
    let quiz = quizzes
        .into_iter()
        .find(|q| q.id == quiz_id)
        .ok_or_else(|| "指定されたクイズが見つかりません".to_string())?;

    let session_id = Uuid::new_v4();
    let join_code = generate_join_code();

    let server_state = Arc::new(ServerState {
        session_id: Some(session_id.to_string()),
        join_code: Arc::new(JoinCodeState::new(join_code.clone())),
        teacher_token: Uuid::new_v4().to_string(),
        active_session: tokio::sync::RwLock::new(Some(session_id)),
        students: tokio::sync::RwLock::new(Vec::new()),
        analysis_events: inference.events.clone(),
        quiz: tokio::sync::RwLock::new(Some(quiz.clone())),
        event_app: Some(app.clone()),
    });

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let handle = start_server_with_state(SESSION_PORT, server_state.clone(), shutdown_rx)
        .await
        .map_err(|e| format!("サーバーの起動に失敗しました: {e}"))?;

    let advertise_handle = match advertise_teacher(
        &app,
        SESSION_PORT,
        &classroom_instance_name(&session_id),
        &session_id.to_string(),
        true,
    )
    .await
    {
        Ok(advertisement) => advertisement,
        Err(error) => {
            let _ = shutdown_tx.send(true);
            let _ = handle.await;
            return Err(format!("教室の通知に失敗しました: {error}"));
        }
    };

    inference.events.lock().await.clear();
    *inference.active_quiz.lock().await = Some(quiz.clone());
    manager.server_state = Some(server_state);
    manager.server_handle = Some(handle);
    manager.server_shutdown = Some(shutdown_tx);
    manager.advertise_handle = Some(advertise_handle.advertise_id);
    manager.active_quiz = Some(quiz);
    manager.session_id = Some(session_id);
    manager.join_code = Some(join_code.clone());

    Ok(join_code)
}

#[tauri::command]
pub async fn end_session(
    app: AppHandle,
    state: State<'_, Mutex<SessionManager>>,
    inference: State<'_, AppState>,
) -> Result<(), String> {
    let mut manager = state.lock().await;

    if let Some(advertise_id) = manager.advertise_handle.take() {
        let _ = stop_advertise(&app, advertise_id).await;
    }

    if let Some(shutdown) = manager.server_shutdown.take() {
        let _ = shutdown.send(true);
    }

    if let Some(handle) = manager.server_handle.take() {
        let _ = handle.await;
    }

    manager.server_state = None;
    manager.active_quiz = None;
    manager.session_id = None;
    manager.join_code = None;
    *inference.active_quiz.lock().await = None;

    Ok(())
}

#[tauri::command]
pub async fn list_students(
    state: State<'_, Mutex<SessionManager>>,
) -> Result<Vec<StudentSummary>, String> {
    let manager = state.lock().await;

    let Some(server_state) = manager.server_state.as_ref() else {
        return Ok(Vec::new());
    };

    let students = server_state.students.read().await;
    Ok(students.iter().map(StudentSummary::from).collect())
}

#[tauri::command]
pub async fn kick_student(
    student_id: String,
    state: State<'_, Mutex<SessionManager>>,
) -> Result<(), String> {
    let manager = state.lock().await;

    let Some(server_state) = manager.server_state.as_ref() else {
        return Err("セッションが開始されていません".into());
    };

    let mut students = server_state.students.write().await;
    students.retain(|s| s.participant_token != student_id);

    Ok(())
}

/// 授業案をセッション中メモリへ採用保存(現行スコープ: 端末内保持)。
#[tauri::command]
pub async fn save_lesson_plan(
    plan_json: String,
    state: tauri::State<'_, Mutex<SessionManager>>,
) -> Result<(), String> {
    let plan: LessonPlan =
        serde_json::from_str(&plan_json).map_err(|e| format!("授業案の解析に失敗: {e}"))?;
    state.lock().await.last_adopted_plan = Some(plan);
    Ok(())
}

/// 直近で採用した授業案を取得(未採用なら null)。
#[tauri::command]
pub async fn get_last_adopted_lesson_plan(
    state: tauri::State<'_, Mutex<SessionManager>>,
) -> Result<Option<LessonPlan>, String> {
    Ok(state.lock().await.last_adopted_plan.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn classroom_instance_name_hides_join_code_and_is_unique() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let name_a = classroom_instance_name(&a);
        let name_b = classroom_instance_name(&b);
        assert!(name_a.starts_with("Lumin教室 "));
        let frag = &name_a["Lumin教室 ".len()..];
        assert_eq!(frag.len(), 6);
        assert!(frag.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(frag, &name_b["Lumin教室 ".len()..]);
    }

    #[test]
    fn student_join_payload_requires_object_token_and_quiz() {
        for invalid in [serde_json::Value::Null, serde_json::json!("unexpected")] {
            assert!(validate_student_join_payload(&mut invalid.clone()).is_err());
        }
        assert!(validate_student_join_payload(&mut serde_json::json!({
            "quiz": {"id": "q"}
        }))
        .is_err());
        assert!(validate_student_join_payload(&mut serde_json::json!({
            "participantToken": "participant-1",
            "quiz": null
        }))
        .is_err());

        let mut valid = serde_json::json!({
            "participantToken": "participant-1",
            "quiz": {"id": "quiz-1"}
        });
        assert_eq!(
            validate_student_join_payload(&mut valid).unwrap(),
            "participant-1"
        );
    }

    #[tokio::test]
    async fn student_result_submission_surfaces_forbidden_and_server_errors() {
        for (status, reason) in [(403, "Forbidden"), (500, "Internal Server Error")] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let server = tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut request = vec![0; 4096];
                let _ = stream.read(&mut request).await.unwrap();
                let response = format!(
                    "HTTP/1.1 {status} {reason}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                );
                stream.write_all(response.as_bytes()).await.unwrap();
            });
            let manager = SessionManager {
                student_connection: Some(StudentConnection {
                    base: format!("http://{address}"),
                    session_id: Uuid::new_v4(),
                    join_code: "1234".into(),
                    participant_token: "participant-1".into(),
                }),
                ..SessionManager::new()
            };
            let event = AnalysisEvent {
                id: Uuid::new_v4(),
                participant_token: String::new(),
                session_id: None,
                question_id: "q1".into(),
                concept: "linear-functions".into(),
                misconception: None,
                correct: true,
                hint_count: 0,
                retry_success: false,
                submitted_at: Utc::now(),
            };

            let error = manager.send_student_analysis(event).await.unwrap_err();
            server.await.unwrap();
            assert!(error.contains(&status.to_string()), "{error}");
            assert!(error.contains("結果を送信できませんでした"));
        }
    }
}
