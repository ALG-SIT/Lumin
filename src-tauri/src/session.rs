use std::collections::HashMap;
use std::sync::Arc;

use tauri::{AppHandle, State};
use tokio::sync::{watch, Mutex};
use uuid::Uuid;

use crate::lumin_core::models::{Quiz, QuizQuestion, StudentInfo};
use crate::network::auth::{generate_join_code, JoinCodeState};
use crate::network::dns_sd::{advertise_teacher, stop_advertise};
use crate::network::server::{start_server_with_state, ServerState};

const SESSION_PORT: u16 = 8765;

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

fn sample_quizzes() -> Vec<Quiz> {
    vec![
        Quiz {
            id: "linear-functions-01".into(),
            title: "一次関数 ミニチェック".into(),
            subject: "中学数学".into(),
            topic: Some("一次関数".into()),
            questions: vec![
                QuizQuestion {
                    id: "math-01".into(),
                    prompt: "y = 3x + 2 の傾きは？".into(),
                    concept: "一次関数の傾き".into(),
                    accepted_answers: vec!["3".into(), "+3".into()],
                    misconception_answers: HashMap::from([("2".into(), "傾きと切片の混同".into())]),
                    generic_misconception: "傾きの読み取り不足".into(),
                    hints: vec![
                        "x が1増えたとき、y がいくつ増えるかに注目しよう。".into(),
                        "y = ax + b の a が変化の割合を表します。".into(),
                    ],
                    explanation: "y = ax + b では、x の係数 a が傾きです。".into(),
                },
                QuizQuestion {
                    id: "math-02".into(),
                    prompt: "y = -2x + 5 の切片は？".into(),
                    concept: "一次関数の切片".into(),
                    accepted_answers: vec!["5".into(), "+5".into()],
                    misconception_answers: HashMap::from([
                        ("-2".into(), "傾きと切片の混同".into()),
                        ("2".into(), "符号の読み落とし".into()),
                    ]),
                    generic_misconception: "切片の読み取り不足".into(),
                    hints: vec![
                        "グラフが y 軸と交わる場所を考えよう。".into(),
                        "x = 0 を式に代入すると切片が分かります。".into(),
                    ],
                    explanation: "x = 0 のとき y = 5 なので、切片は5です。".into(),
                },
            ],
        },
        Quiz {
            id: "english-grammar-01".into(),
            title: "英語文法チェック".into(),
            subject: "英語".into(),
            topic: Some("過去形".into()),
            questions: vec![QuizQuestion {
                id: "eng-01".into(),
                prompt: "Yesterday she _____ to the store.".into(),
                concept: "past tense".into(),
                accepted_answers: vec!["went".into()],
                misconception_answers: HashMap::from([("go".into(), "past tense error".into())]),
                generic_misconception: "verb tense error".into(),
                hints: vec![
                    "What is the past tense of 'go'?".into(),
                    "Think about irregular verbs.".into(),
                ],
                explanation: "The past tense of 'go' is 'went'.".into(),
            }],
        },
        Quiz {
            id: "japanese-grammar-01".into(),
            title: "日本語文法テスト".into(),
            subject: "日本語".into(),
            topic: Some("助詞".into()),
            questions: vec![QuizQuestion {
                id: "jpn-01".into(),
                prompt: "学校＿＿＿行きます。".into(),
                concept: "格助詞".into(),
                accepted_answers: vec!["に".into()],
                misconception_answers: HashMap::new(),
                generic_misconception: "助詞の使い分け不足".into(),
                hints: vec![
                    "場所を表す格助詞を考えよう。".into(),
                    "移動の目的地には「に」を使う。".into(),
                ],
                explanation: "移動の目的地には「に」を使います。".into(),
            }],
        },
        Quiz {
            id: "science-density-01".into(),
            title: "密度と体積".into(),
            subject: "理科".into(),
            topic: Some("密度".into()),
            questions: vec![QuizQuestion {
                id: "sci-01".into(),
                prompt: "質量100g、体積50cm³の物体の密度は？".into(),
                concept: "密度の計算".into(),
                accepted_answers: vec!["2".into(), "2g/cm3".into(), "2g/cm³".into()],
                misconception_answers: HashMap::from([("0.5".into(), "質量と体積の逆転".into())]),
                generic_misconception: "密度の公式の誤用".into(),
                hints: vec![
                    "密度 = 質量 ÷ 体積 です。".into(),
                    "100 ÷ 50 を計算してみよう。".into(),
                ],
                explanation: "密度 = 質量 ÷ 体積 = 100 ÷ 50 = 2 g/cm³ です。".into(),
            }],
        },
        Quiz {
            id: "social-history-01".into(),
            title: "明治維新の要点".into(),
            subject: "社会".into(),
            topic: Some("日本史".into()),
            questions: vec![QuizQuestion {
                id: "soc-01".into(),
                prompt: "明治維新が起こった年は？".into(),
                concept: "明治維新".into(),
                accepted_answers: vec!["1868".into()],
                misconception_answers: HashMap::from([("1867".into(), "大政奉還と混同".into())]),
                generic_misconception: "年号と西暦の混同".into(),
                hints: vec![
                    "江戸時代が終わり、明治時代が始まった年です。".into(),
                    "大政奉還の翌年に政府が発足しました。".into(),
                ],
                explanation: "1868年に明治政府が成立し、明治維新が始まりました。".into(),
            }],
        },
    ]
}

#[tauri::command]
pub async fn list_quizzes() -> Result<Vec<Quiz>, String> {
    Ok(sample_quizzes())
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

    let quizzes = sample_quizzes();
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
        &format!("Lumin教室 {}", join_code),
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
