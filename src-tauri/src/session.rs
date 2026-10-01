use std::collections::VecDeque;
use std::sync::Arc;

use tauri::{AppHandle, State};
use tokio::{
    net::TcpStream,
    sync::{watch, Mutex},
};
use uuid::Uuid;

use crate::inference::session::AppState;
use crate::lumin_core::models::{AnalysisEvent, Quiz, StudentInfo};
use crate::lumin_core::LessonPlan;
use crate::network::auth::{generate_join_code, JoinCodeState};
use crate::network::dns_sd::{advertise_teacher, stop_advertise};
use crate::network::noise::{
    self, Connection as NoiseConnection, Request as NoiseRequest, Response as NoiseResponse,
};
use crate::network::server::{start_server_with_state, ServerState};

type EncryptedTcpConnection = Arc<Mutex<NoiseConnection<TcpStream>>>;
type PendingEventQueue = Arc<Mutex<VecDeque<AnalysisEvent>>>;
type PendingStudentConnection = (String, String, Uuid, String, u16, EncryptedTcpConnection);

const SESSION_PORT: u16 = 8765;

/// 教室(mDNSインスタンス)名。参加コードは秘匿情報のため名前に含めない。
/// 一意性確保のためセッションUUID先頭6桁(大文字hex)を添える。
fn classroom_instance_name(session_id: &Uuid) -> String {
    let hex = session_id.simple().to_string();
    format!("Lumin教室 {}", hex[..6].to_uppercase())
}

#[cfg(test)]
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
#[serde(rename_all = "camelCase")]
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
    pending_connection: Option<PendingStudentConnection>,
    teacher_fingerprint: Option<String>,
    student_private_key: Option<Vec<u8>>,
    pending_events: PendingEventQueue,
}

#[derive(Clone)]
pub(crate) struct StudentConnection {
    session_id: Uuid,
    participant_token: String,
    connection: Arc<Mutex<NoiseConnection<TcpStream>>>,
    host: String,
    port: u16,
    join_code: String,
    private_key: Vec<u8>,
    teacher_fingerprint: String,
}

async fn send_with_reconnect(
    connection: &StudentConnection,
    event: AnalysisEvent,
) -> Result<(), String> {
    let event_id = event.id;
    let result = {
        let mut socket = connection.connection.lock().await;
        async {
            socket.send(&NoiseRequest::Analysis(event.clone())).await?;
            match socket.receive::<NoiseResponse>().await? {
                NoiseResponse::Ack(ack) if ack.event_id == event_id => Ok(()),
                NoiseResponse::Error { message } => Err(message),
                _ => Err("教師端末から不正な応答がありました".into()),
            }
        }
        .await
    };
    if result.is_ok() {
        return result;
    }

    let socket = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        TcpStream::connect((connection.host.as_str(), connection.port)),
    )
    .await
    .map_err(|_| "教師端末への再接続がタイムアウトしました".to_string())?
    .map_err(|e| format!("再接続できませんでした: {e}"))?;
    socket.set_nodelay(true).map_err(|e| e.to_string())?;
    let mut reconnected =
        NoiseConnection::handshake_client(socket, &connection.private_key).await?;
    if noise::fingerprint(&reconnected.remote_static) != connection.teacher_fingerprint {
        return Err("教師端末の鍵が変わりました。安全のため再送を停止しました".into());
    }
    reconnected.send(&NoiseRequest::Prepare).await?;
    if !matches!(reconnected.receive::<NoiseResponse>().await?, NoiseResponse::Prepared { session_id } if session_id == connection.session_id)
    {
        return Err("同じ授業へ再接続できませんでした".into());
    }
    reconnected
        .send(&NoiseRequest::Join {
            session_id: connection.session_id,
            join_code: connection.join_code.clone(),
        })
        .await?;
    match reconnected.receive::<NoiseResponse>().await? {
        NoiseResponse::Joined {
            session_id,
            participant_token,
            ..
        } if session_id == connection.session_id
            && participant_token == connection.participant_token => {}
        _ => return Err("参加状態を復元できませんでした".into()),
    }
    reconnected.send(&NoiseRequest::Analysis(event)).await?;
    match reconnected.receive::<NoiseResponse>().await? {
        NoiseResponse::Ack(ack) if ack.event_id == event_id => {
            *connection.connection.lock().await = reconnected;
            Ok(())
        }
        NoiseResponse::Error { message } => Err(message),
        _ => Err("教師端末から不正な応答がありました".into()),
    }
}

impl SessionManager {
    pub(crate) fn student_send_context(
        &self,
    ) -> Result<(StudentConnection, PendingEventQueue), String> {
        let connection = self
            .student_connection
            .as_ref()
            .ok_or("教室に参加し直してください")?
            .clone();
        Ok((connection, self.pending_events.clone()))
    }
}

pub(crate) async fn send_queued_analysis(
    connection: StudentConnection,
    pending: Arc<Mutex<VecDeque<AnalysisEvent>>>,
    mut event: AnalysisEvent,
) -> Result<(), String> {
    event.session_id = Some(connection.session_id);
    event.participant_token = connection.participant_token.clone();
    let mut pending = pending.lock().await;
    if !pending.iter().any(|queued| queued.id == event.id) {
        if pending.len() >= 1024 {
            return Err(
                "送信待ちの回答が上限に達しました。通信を復旧してから再度お試しください".into(),
            );
        }
        pending.push_back(event);
    }
    while let Some(next) = pending.front().cloned() {
        let mut delivered = false;
        let mut last_error = String::new();
        for attempt in 0..5 {
            match send_with_reconnect(&connection, next.clone()).await {
                Ok(()) => {
                    delivered = true;
                    break;
                }
                Err(error) => {
                    last_error = error;
                    if attempt < 4 {
                        tokio::time::sleep(std::time::Duration::from_millis(
                            [500, 1000, 2000, 4000, 8000][attempt],
                        ))
                        .await;
                    }
                }
            }
        }
        if !delivered {
            return Err(format!("回答を送信待ちに保存しました: {last_error}"));
        }
        pending.pop_front();
    }
    Ok(())
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
            pending_connection: None,
            teacher_fingerprint: None,
            student_private_key: None,
            pending_events: Arc::new(Mutex::new(VecDeque::new())),
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
    teacher_fingerprint: Option<String>,
}

#[tauri::command]
pub async fn get_teacher_session(
    state: State<'_, Mutex<SessionManager>>,
) -> Result<TeacherSessionSnapshot, String> {
    let manager = state.lock().await;
    Ok(TeacherSessionSnapshot {
        quiz: manager.active_quiz.clone(),
        join_code: manager.join_code.clone(),
        teacher_fingerprint: manager.teacher_fingerprint.clone(),
    })
}

#[tauri::command]
pub async fn list_quizzes() -> Result<Vec<Quiz>, String> {
    use crate::lumin_core::demo_data;
    Ok(demo_data::demo_quiz_bank())
}

/// TCP＋Noise 接続を準備し、教師の鍵確認文字列を返す。
#[tauri::command]
pub async fn student_prepare_connection(
    host: String,
    port: u16,
    session_id: Option<String>,
    manager: State<'_, Mutex<SessionManager>>,
) -> Result<serde_json::Value, String> {
    if host.is_empty() || port == 0 {
        return Err("接続先が正しくありません".into());
    }
    let expected_id = session_id
        .filter(|s| !s.is_empty())
        .map(|s| Uuid::parse_str(&s).map_err(|e| e.to_string()))
        .transpose()?;
    let private_key = {
        let mut manager = manager.lock().await;
        if manager.student_private_key.is_none() {
            manager.student_private_key = Some(noise::keypair()?.private);
        }
        manager
            .student_private_key
            .clone()
            .ok_or("生徒の鍵を生成できません")?
    };
    let socket = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        TcpStream::connect((host.as_str(), port)),
    )
    .await
    .map_err(|_| "教師端末への接続がタイムアウトしました".to_string())?
    .map_err(|e| format!("教師端末へ接続できませんでした: {e}"))?;
    socket.set_nodelay(true).map_err(|e| e.to_string())?;
    let mut connection = NoiseConnection::handshake_client(socket, &private_key)
        .await
        .map_err(|e| {
            format!("教師端末と安全に接続できませんでした。両方のアプリを更新してください: {e}")
        })?;
    let teacher_fingerprint = noise::fingerprint(&connection.remote_static);
    connection.send(&NoiseRequest::Prepare).await?;
    let session_id = match connection.receive::<NoiseResponse>().await? {
        NoiseResponse::Prepared { session_id }
            if expected_id.is_none_or(|expected| expected == session_id) =>
        {
            session_id
        }
        NoiseResponse::Prepared { .. } => {
            return Err("発見情報と教室のセッションが一致しません".into())
        }
        NoiseResponse::Error { message } => return Err(message),
        _ => return Err("教室の情報を取得できませんでした".into()),
    };
    let mut manager = manager.lock().await;
    let pending_id = Uuid::new_v4().to_string();
    manager.pending_connection = Some((
        pending_id.clone(),
        teacher_fingerprint.clone(),
        session_id,
        host,
        port,
        Arc::new(Mutex::new(connection)),
    ));
    Ok(
        serde_json::json!({"pendingId": pending_id, "teacherFingerprint": teacher_fingerprint, "sessionId": session_id}),
    )
}

#[tauri::command]
pub async fn student_cancel_connection(
    manager: State<'_, Mutex<SessionManager>>,
) -> Result<(), String> {
    let mut manager = manager.lock().await;
    manager.pending_connection = None;
    Ok(())
}

/// 学生が確認した教師の鍵に対して参加コードを送る。
#[tauri::command]
pub async fn student_join(
    pending_id: String,
    teacher_fingerprint: String,
    join_code: String,
    manager: tauri::State<'_, Mutex<SessionManager>>,
) -> Result<serde_json::Value, String> {
    if join_code.len() != 4 || !join_code.chars().all(|c| c.is_ascii_digit()) {
        return Err("参加コードは4桁の数字で入力してください".into());
    }
    let mut manager = manager.lock().await;
    let (id, fingerprint, session_id, host, port, connection_ref) = manager
        .pending_connection
        .take()
        .ok_or("教室へ接続し直してください")?;
    if id != pending_id || fingerprint != teacher_fingerprint {
        return Err("教師の確認文字列が一致しません".into());
    }
    let mut connection = connection_ref.lock().await;
    connection
        .send(&NoiseRequest::Join {
            session_id,
            join_code: join_code.clone(),
        })
        .await?;
    let (token, quiz) = match connection.receive::<NoiseResponse>().await? {
        NoiseResponse::Joined {
            session_id: joined,
            participant_token,
            quiz,
        } if joined == session_id => (participant_token, quiz),
        NoiseResponse::Error { message } => return Err(message),
        _ => return Err("参加情報が不正です".into()),
    };
    let private_key = manager
        .student_private_key
        .clone()
        .ok_or("生徒の鍵を生成できません")?;
    let connection_ref = connection_ref.clone();
    manager.student_connection = Some(StudentConnection {
        session_id,
        participant_token: token.clone(),
        connection: connection_ref.clone(),
        host,
        port,
        join_code,
        private_key,
        teacher_fingerprint: fingerprint,
    });
    noise::start_heartbeat(&connection_ref);
    Ok(serde_json::json!({"participantToken": token, "sessionId": session_id, "quiz": quiz}))
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

    let (handle, teacher_fingerprint) =
        start_server_with_state(SESSION_PORT, server_state.clone(), shutdown_rx)
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
    manager.teacher_fingerprint = Some(teacher_fingerprint);

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
    manager.teacher_fingerprint = None;
    manager.student_connection = None;
    manager.pending_connection = None;
    manager.student_private_key = None;
    manager.pending_events.lock().await.clear();
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
}
