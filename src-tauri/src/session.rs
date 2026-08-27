use std::sync::Arc;

use axum::http::StatusCode;

use tauri::{AppHandle, State};
use tokio::sync::{watch, Mutex};
use uuid::Uuid;

use crate::lumin_core::models::{Quiz, StudentInfo};
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
    let mut req = client
        .post(format!("{base}/students/join"))
        .header("X-Lumin-Join-Code", &join_code);
    // 発見経路ではセッションUUID付き。手動入力(sessionId="")では省略。
    let session_id = session_id.filter(|s| !s.is_empty());
    if let Some(id) = session_id {
        req = req.header("X-Lumin-Session-ID", id);
    }

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
            if let Some(obj) = body.as_object_mut() {
                obj.insert("host".into(), serde_json::Value::String(host));
                obj.insert("port".into(), serde_json::Value::from(port));
                // 配信中クイズを学生へ届ける(main実装の activeQuiz 相当)
                let manager = _manager.lock().await;
                obj.insert(
                    "quiz".to_string(),
                    manager
                        .active_quiz
                        .clone()
                        .map_or(serde_json::Value::Null, |q| {
                            serde_json::to_value(q).expect("quiz serializable")
                        }),
                );
            }
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
        analysis_events: tokio::sync::RwLock::new(Vec::new()),
    });

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let handle = start_server_with_state(SESSION_PORT, server_state.clone(), shutdown_rx)
        .await
        .map_err(|e| format!("サーバーの起動に失敗しました: {e}"))?;

    let advertise_handle = advertise_teacher(
        &app,
        SESSION_PORT,
        &classroom_instance_name(&session_id),
        &session_id.to_string(),
        true,
    )
    .await
    .map_err(|e| format!("教室の通知に失敗しました: {e}"))?;

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
}
