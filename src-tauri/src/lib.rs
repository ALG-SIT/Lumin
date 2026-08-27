mod ai;
mod inference;
mod lumin_core;
mod network;
pub mod persistence;
mod session;

use crate::network::dns_sd::browse_teachers;
use chrono::Utc;
use inference::generate::{GenerateOptions, GenerateResult};
use inference::model_download::DownloadResult;
use inference::session::{resolve_model_dir, AppState, ModelInfo};
use lumin_core::analyzer::ClassAnalytics;
use lumin_core::{AnalysisEvent, ClassSummary, Quiz, QuizQuestion};
use serde::{Deserialize, Serialize};
use session::SessionManager;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use tauri::{Emitter, Manager, State};
use tokio::sync::Mutex;
use uuid::Uuid;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub platform: String,
    pub arch: String,
    pub tauri_version: String,
    pub ort_available: bool,
    pub model_dir: String,
}

#[tauri::command]
async fn get_system_info(state: State<'_, AppState>) -> Result<SystemInfo, String> {
    Ok(SystemInfo {
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        tauri_version: "2".to_string(),
        ort_available: true,
        model_dir: state.model_dir.to_string_lossy().to_string(),
    })
}

#[tauri::command]
async fn check_model_status(state: State<'_, AppState>) -> Result<Vec<ModelInfo>, String> {
    Ok(state.model_variants())
}

#[tauri::command]
async fn get_model_info(state: State<'_, AppState>) -> Result<Vec<ModelInfo>, String> {
    Ok(state.model_variants())
}

#[tauri::command]
async fn generate(
    prompt: String,
    max_tokens: Option<usize>,
    temperature: Option<f32>,
    use_chat_template: Option<bool>,
    state: State<'_, AppState>,
) -> Result<GenerateResult, String> {
    let opts = GenerateOptions {
        prompt,
        max_tokens,
        temperature,
        use_chat_template,
    };
    inference::generate::generate_text(&state, opts)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn generate_stream(
    app: tauri::AppHandle,
    prompt: String,
    max_tokens: Option<usize>,
    temperature: Option<f32>,
    use_chat_template: Option<bool>,
    state: State<'_, AppState>,
) -> Result<GenerateResult, String> {
    let opts = GenerateOptions {
        prompt,
        max_tokens,
        temperature,
        use_chat_template,
    };

    let result = inference::generate::generate_stream(&state, opts, |token| {
        if let Err(e) = app.emit("token", token) {
            eprintln!("[emit] token failed: {e}");
        }
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?;

    if let Err(e) = app.emit("generation-complete", &result) {
        eprintln!("[emit] generation-complete failed: {e}");
    }
    Ok(result)
}

#[tauri::command]
async fn bench_inference(
    iterations: Option<usize>,
    state: State<'_, AppState>,
) -> Result<inference::bench::BenchResult, String> {
    let iters = iterations.unwrap_or(3).min(10);
    inference::bench::run_bench(&state, iters)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn download_model(
    app: tauri::AppHandle,
    variant: Option<String>,
    state: State<'_, AppState>,
) -> Result<DownloadResult, String> {
    let v = variant.unwrap_or_else(|| "1b-int4".to_string());
    state.download_cancelled.store(false, Ordering::SeqCst);
    inference::model_download::download_model_with_progress(
        app,
        state.model_dir.clone(),
        v,
        state.download_cancelled.clone(),
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn send_analysis_event(
    app: tauri::AppHandle,
    event_json: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let event: AnalysisEvent = serde_json::from_str(&event_json).map_err(|e| e.to_string())?;
    {
        let mut events = state.events.lock().await;
        events.push(event.clone());
    }
    if let Err(e) = app.emit("analysis-event", &event) {
        eprintln!("[emit] analysis-event failed: {e}");
    }
    Ok(())
}

#[tauri::command]
async fn get_class_summary(state: State<'_, AppState>) -> Result<ClassSummary, String> {
    let events = state.events.lock().await;
    Ok(ClassAnalytics::summarize(&events))
}

fn demo_quiz() -> Quiz {
    let mut misconception_answers = HashMap::new();
    misconception_answers.insert("2".into(), "傾きと切片の混同".into());
    Quiz {
        id: "demo-quiz".into(),
        title: "一次関数 ミニチェック".into(),
        subject: "中学数学".into(),
        topic: Some("一次関数".into()),
        questions: vec![QuizQuestion {
            id: "math-01".into(),
            prompt: "y = 3x + 2 の傾きは？".into(),
            concept: "一次関数の傾き".into(),
            accepted_answers: vec!["3".into(), "+3".into()],
            misconception_answers,
            generic_misconception: "傾きの読み取り不足".into(),
            hints: vec![
                "x が1増えたとき、y がいくつ増えるかに注目しよう。".into(),
                "y = ax + b の a が変化の割合を表します。".into(),
                "この式を y = a×x + b と見比べ、x の直前の数を探そう。".into(),
            ],
            explanation: "y = ax + b では、x の係数 a が傾きです。".into(),
        }],
    }
}

fn demo_event(
    participant: &str,
    question_id: &str,
    concept: &str,
    misconception: Option<&str>,
    correct: bool,
    hint_count: i32,
    retry_success: bool,
) -> AnalysisEvent {
    AnalysisEvent {
        id: Uuid::new_v4(),
        participant_token: participant.into(),
        session_id: None,
        question_id: question_id.into(),
        concept: concept.into(),
        misconception: misconception.map(|s| s.into()),
        correct,
        hint_count,
        retry_success,
        submitted_at: Utc::now(),
    }
}

fn demo_events() -> Vec<AnalysisEvent> {
    vec![
        demo_event(
            "P-1",
            "math-01",
            "一次関数の傾き",
            Some("傾きと切片の混同"),
            false,
            2,
            true,
        ),
        demo_event("P-1", "math-02", "一次関数の切片", None, true, 0, false),
        demo_event(
            "P-1",
            "math-03",
            "変化の割合",
            Some("変化量の分子と分母の逆転"),
            false,
            1,
            true,
        ),
        demo_event("P-2", "math-01", "一次関数の傾き", None, true, 0, false),
        demo_event(
            "P-2",
            "math-04",
            "式への代入",
            Some("切片の計算漏れ"),
            false,
            3,
            true,
        ),
        demo_event(
            "P-2",
            "math-05",
            "一次関数の式",
            Some("傾きと切片の混同"),
            false,
            2,
            false,
        ),
        demo_event(
            "P-3",
            "math-01",
            "一次関数の傾き",
            Some("傾きと切片の混同"),
            false,
            1,
            true,
        ),
        demo_event("P-3", "math-02", "一次関数の切片", None, true, 0, false),
        demo_event("P-3", "math-03", "変化の割合", None, true, 0, false),
        demo_event(
            "P-4",
            "math-04",
            "式への代入",
            Some("代入計算の誤り"),
            false,
            2,
            true,
        ),
        demo_event("P-4", "math-05", "一次関数の式", None, true, 0, false),
        demo_event("P-5", "math-01", "一次関数の傾き", None, true, 0, false),
        demo_event(
            "P-5",
            "math-02",
            "一次関数の切片",
            Some("傾きと切片の混同"),
            false,
            3,
            false,
        ),
        demo_event(
            "P-6",
            "math-03",
            "変化の割合",
            Some("変化量の分子と分母の逆転"),
            false,
            1,
            true,
        ),
        demo_event("P-6", "math-04", "式への代入", None, true, 0, false),
        demo_event("P-6", "math-05", "一次関数の式", None, true, 0, false),
        demo_event(
            "P-7",
            "math-01",
            "一次関数の傾き",
            Some("傾きと切片の混同"),
            false,
            2,
            true,
        ),
        demo_event("P-7", "math-02", "一次関数の切片", None, true, 0, false),
        demo_event("P-8", "math-03", "変化の割合", None, true, 0, false),
        demo_event(
            "P-8",
            "math-05",
            "一次関数の式",
            Some("傾きの符号の読み落とし"),
            false,
            4,
            true,
        ),
        demo_event(
            "P-9",
            "math-01",
            "一次関数の傾き",
            Some("傾きと切片の混同"),
            false,
            1,
            false,
        ),
        demo_event("P-9", "math-04", "式への代入", None, true, 0, false),
        demo_event(
            "P-10",
            "math-02",
            "一次関数の切片",
            Some("傾きと切片の混同"),
            false,
            3,
            true,
        ),
        demo_event("P-10", "math-03", "変化の割合", None, true, 0, false),
        demo_event(
            "P-10",
            "math-05",
            "一次関数の式",
            Some("傾きの符号の読み落とし"),
            false,
            2,
            true,
        ),
    ]
}

#[tauri::command]
async fn load_demo_data(state: State<'_, AppState>) -> Result<(), String> {
    let events = demo_events();
    {
        let mut store = state.events.lock().await;
        store.clear();
        store.extend(events);
    }
    {
        let mut quiz = state.active_quiz.lock().await;
        *quiz = Some(demo_quiz());
    }
    Ok(())
}

#[tauri::command]
async fn start_demo_session(state: State<'_, AppState>) -> Result<String, String> {
    use lumin_core::demo_data;
    let quiz = demo_data::demo_quiz_short();
    let session_id = Uuid::new_v4();
    {
        let mut q = state.active_quiz.lock().await;
        *q = Some(quiz.clone());
    }
    {
        let mut events = state.events.lock().await;
        events.clear();
    }
    Ok(session_id.to_string())
}

#[tauri::command]
async fn submit_demo_events(state: State<'_, AppState>) -> Result<(), String> {
    use lumin_core::demo_data;
    let events = demo_data::demo_events();
    let mut store = state.events.lock().await;
    store.clear();
    store.extend(events);
    Ok(())
}

#[tauri::command]
async fn get_demo_quiz_summary(state: State<'_, AppState>) -> Result<ClassSummary, String> {
    let events = state.events.lock().await;
    Ok(ClassAnalytics::summarize(&events))
}

#[tauri::command]
async fn get_demo_lesson_plan(
    state: State<'_, AppState>,
) -> Result<lumin_core::LessonPlan, String> {
    use lumin_core::analyzer::{ClassAnalytics, LessonPlanGenerator};
    let events = state.events.lock().await;
    let summary = ClassAnalytics::summarize(&events);
    let quiz = state.active_quiz.lock().await;
    Ok(LessonPlanGenerator::generate(&summary, quiz.as_ref()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = ort::init().commit();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_dns_sd::init())
        .setup(|app| {
            let model_dir = resolve_model_dir_for_app(app.handle());
            let _ = std::fs::create_dir_all(&model_dir);
            app.manage(AppState::new(model_dir));
            app.manage(Mutex::new(SessionManager::new()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_system_info,
            check_model_status,
            get_model_info,
            generate,
            generate_stream,
            bench_inference,
            download_model,
            inference::list_models,
            inference::import_model,
            inference::cancel_download,
            ai::commands::analyze_answer,
            ai::commands::generate_hint,
            ai::commands::generate_lesson_plan,
            ai::commands::chat_with_teacher,
            send_analysis_event,
            get_class_summary,
            load_demo_data,
            start_demo_session,
            submit_demo_events,
            get_demo_quiz_summary,
            get_demo_lesson_plan,
            browse_teachers,
            session::student_join,
            session::list_quizzes,
            session::start_session,
            session::end_session,
            session::list_students,
            session::kick_student,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn resolve_model_dir_for_app(app: &tauri::AppHandle) -> std::path::PathBuf {
    if let Ok(app_data) = app.path().app_data_dir() {
        let candidate = app_data.join("models");
        let project_models = resolve_model_dir();
        let use_project = project_models.exists() && cfg!(debug_assertions) && !is_mobile(app);
        if use_project {
            return project_models;
        }
        return candidate;
    }
    resolve_model_dir()
}

fn is_mobile(_app: &tauri::AppHandle) -> bool {
    #[cfg(mobile)]
    {
        true
    }
    #[cfg(not(mobile))]
    {
        false
    }
}
