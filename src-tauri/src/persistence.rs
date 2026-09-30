#![allow(dead_code)]

use crate::lumin_core::models::{AnalysisEvent, LearningSession, SessionArchive};
use std::path::PathBuf;
use tokio::fs;

const HISTORY_LIMIT: usize = 100;

/// Directory layout:
///   app_data_dir/
///     session-history.json      (Vec<SessionArchive>, cap 100)
///     active-session-recovery.json (Option<LearningSession>, teacher crash recovery)
///     pending-analysis.json     (Vec<AnalysisEvent>, student side — no answer/hint text)
pub struct Persistence {
    dir: PathBuf,
}

impl Persistence {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    // ── Session history ────────────────────────────────────────────────

    /// Load the full history. Returns an empty vec if the file is missing.
    pub async fn load_history(&self) -> Result<Vec<SessionArchive>, String> {
        let bytes = match fs::read(self.history_path()).await {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(format!("history read: {e}")),
        };
        serde_json::from_slice(&bytes).map_err(|e| format!("history decode: {e}"))
    }

    /// Prepend a new session and enforce the 100-item cap.
    pub async fn append_session(&self, session: SessionArchive) -> Result<(), String> {
        let mut history = self.load_history().await?;
        history.insert(0, session);
        history.truncate(HISTORY_LIMIT);
        self.atomic_write(&self.history_path(), &history).await
    }

    // ── Active session recovery (teacher crash recovery) ───────────────

    /// Recover the active session, if any.
    pub async fn load_active_session(&self) -> Result<Option<LearningSession>, String> {
        let bytes = match fs::read(self.active_path()).await {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("active read: {e}")),
        };
        let session: LearningSession =
            serde_json::from_slice(&bytes).map_err(|e| format!("active decode: {e}"))?;
        Ok(Some(session))
    }

    /// Persist the active session. Passing `None` deletes the file.
    pub async fn save_active_session(
        &self,
        session: Option<&LearningSession>,
    ) -> Result<(), String> {
        match session {
            Some(s) => self.atomic_write_raw(&self.active_path(), s).await,
            None => {
                let _ = fs::remove_file(self.active_path()).await;
                Ok(())
            }
        }
    }

    // ── Pending analysis queue (student side) ──────────────────────────

    /// Load the pending event queue. Returns an empty vec if missing.
    pub async fn load_pending(&self) -> Result<Vec<AnalysisEvent>, String> {
        let bytes = match fs::read(self.pending_path()).await {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(format!("pending read: {e}")),
        };
        serde_json::from_slice(&bytes).map_err(|e| format!("pending decode: {e}"))
    }

    /// Replace the pending queue (crash-safe via temp file + rename).
    pub async fn save_pending(&self, events: &[AnalysisEvent]) -> Result<(), String> {
        self.atomic_write_raw(&self.pending_path(), events).await
    }

    // ── Internals ──────────────────────────────────────────────────────

    fn history_path(&self) -> PathBuf {
        self.dir.join("session-history.json")
    }
    fn active_path(&self) -> PathBuf {
        self.dir.join("active-session-recovery.json")
    }
    fn pending_path(&self) -> PathBuf {
        self.dir.join("pending-analysis.json")
    }

    /// Atomic write: serialize to a `.tmp` file, then `fs::rename` over the target.
    async fn atomic_write<T: serde::Serialize>(
        &self,
        path: &PathBuf,
        value: &T,
    ) -> Result<(), String> {
        let json = serde_json::to_vec_pretty(value).map_err(|e| format!("serialize: {e}"))?;
        self.write_bytes(path, &json).await
    }

    async fn atomic_write_raw<T: serde::Serialize + ?Sized>(
        &self,
        path: &PathBuf,
        value: &T,
    ) -> Result<(), String> {
        let json = serde_json::to_vec_pretty(value).map_err(|e| format!("serialize: {e}"))?;
        self.write_bytes(path, &json).await
    }

    async fn write_bytes(&self, path: &PathBuf, bytes: &[u8]) -> Result<(), String> {
        // Ensure parent directory exists.
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("mkdir: {e}"))?;
        }
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, bytes)
            .await
            .map_err(|e| format!("tmp write: {e}"))?;
        fs::rename(&tmp, path)
            .await
            .map_err(|e| format!("rename: {e}"))?;
        Ok(())
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::tempdir;
    use uuid::Uuid;

    fn sample_quiz() -> crate::lumin_core::models::Quiz {
        use std::collections::HashMap;
        let mut misconceptions = HashMap::new();
        misconceptions.insert("1/2".into(), "added numerators".into());
        crate::lumin_core::models::Quiz {
            id: "quiz1".into(),
            title: "Fractions Quiz".into(),
            subject: "Math".into(),
            topic: Some("Fractions".into()),
            questions: vec![crate::lumin_core::models::QuizQuestion {
                id: "q1".into(),
                prompt: "What is 1/2 + 1/3?".into(),
                concept: "fraction addition".into(),
                accepted_answers: vec!["5/6".into()],
                misconception_answers: misconceptions,
                generic_misconception: "incorrect fraction addition".into(),
                hints: vec!["Find a common denominator".into()],
                explanation: "Convert to 3/6 + 2/6 = 5/6".into(),
            }],
        }
    }

    fn sample_session() -> LearningSession {
        LearningSession {
            id: Uuid::new_v4(),
            quiz: sample_quiz(),
            started_at: Utc::now(),
        }
    }

    fn sample_archive(session: &LearningSession) -> SessionArchive {
        SessionArchive {
            id: Uuid::new_v4(),
            session: session.clone(),
            ended_at: Utc::now(),
            events: vec![],
            adopted_plan: None,
        }
    }

    fn sample_event() -> AnalysisEvent {
        AnalysisEvent {
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
        }
    }

    // ── History tests ──────────────────────────────────────────────────

    #[tokio::test]
    async fn test_history_load_empty_when_missing() {
        let tmp = tempdir().unwrap();
        let p = Persistence::new(tmp.path().into());
        let h = p.load_history().await.unwrap();
        assert!(h.is_empty());
    }

    #[tokio::test]
    async fn corrupt_history_returns_error_without_changing_source_file() {
        let dir = tempdir().unwrap();
        let persistence = Persistence::new(dir.path().into());
        let path = dir.path().join("session-history.json");
        let corrupt = b"{broken json";
        fs::write(&path, corrupt).await.unwrap();

        assert!(persistence.load_history().await.is_err());
        assert_eq!(fs::read(path).await.unwrap(), corrupt);
    }

    #[tokio::test]
    async fn persistence_returns_read_and_write_errors_instead_of_succeeding() {
        let dir = tempdir().unwrap();
        let blocker = dir.path().join("not-a-directory");
        fs::write(&blocker, b"keep this file").await.unwrap();
        let persistence = Persistence::new(blocker.clone());

        assert!(persistence.load_history().await.is_err());
        assert!(persistence.save_pending(&[]).await.is_err());
        assert_eq!(fs::read(blocker).await.unwrap(), b"keep this file");
    }

    #[tokio::test]
    async fn test_history_append_and_retrieve() {
        let tmp = tempdir().unwrap();
        let p = Persistence::new(tmp.path().into());
        let session = sample_session();
        let archive = sample_archive(&session);
        p.append_session(archive.clone()).await.unwrap();

        let h = p.load_history().await.unwrap();
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].id, archive.id);
    }

    #[tokio::test]
    async fn test_history_cap_100() {
        let tmp = tempdir().unwrap();
        let p = Persistence::new(tmp.path().into());

        // Insert 150 sessions in reverse chronological order (newest first)
        // so we can verify the last 50 are dropped.
        let mut ids = Vec::new();
        for _ in 0..150 {
            let s = sample_session();
            let a = sample_archive(&s);
            ids.push(a.id);
            p.append_session(a).await.unwrap();
        }

        let h = p.load_history().await.unwrap();
        assert_eq!(h.len(), 100);

        // The most recently appended session should be first.
        assert_eq!(h[0].id, ids[149]);
        // The 50 oldest should have been truncated.
        assert_eq!(h[99].id, ids[50]);
    }

    #[tokio::test]
    async fn test_history_truncates_existing() {
        // If the file already contains 100 entries and we append one more,
        // we still end up with 100.
        let tmp = tempdir().unwrap();
        let p = Persistence::new(tmp.path().into());

        for _ in 0..100 {
            let s = sample_session();
            let a = sample_archive(&s);
            p.append_session(a).await.unwrap();
        }

        let extra = sample_archive(&sample_session());
        p.append_session(extra.clone()).await.unwrap();

        let h = p.load_history().await.unwrap();
        assert_eq!(h.len(), 100);
        assert_eq!(h[0].id, extra.id);
    }

    // ── Active session recovery tests ──────────────────────────────────

    #[tokio::test]
    async fn test_active_session_none_when_missing() {
        let tmp = tempdir().unwrap();
        let p = Persistence::new(tmp.path().into());
        let s = p.load_active_session().await.unwrap();
        assert!(s.is_none());
    }

    #[tokio::test]
    async fn test_active_session_recovery() {
        let tmp = tempdir().unwrap();
        let p = Persistence::new(tmp.path().into());
        let session = sample_session();

        p.save_active_session(Some(&session)).await.unwrap();
        let recovered = p.load_active_session().await.unwrap().unwrap();
        assert_eq!(recovered.id, session.id);
        assert_eq!(recovered.quiz.id, session.quiz.id);
    }

    #[tokio::test]
    async fn test_active_session_delete() {
        let tmp = tempdir().unwrap();
        let p = Persistence::new(tmp.path().into());
        let session = sample_session();

        p.save_active_session(Some(&session)).await.unwrap();
        assert!(p.load_active_session().await.unwrap().is_some());

        p.save_active_session(None).await.unwrap();
        assert!(p.load_active_session().await.unwrap().is_none());
    }

    // ── Pending queue tests ────────────────────────────────────────────

    #[tokio::test]
    async fn test_pending_empty_when_missing() {
        let tmp = tempdir().unwrap();
        let p = Persistence::new(tmp.path().into());
        let q = p.load_pending().await.unwrap();
        assert!(q.is_empty());
    }

    #[tokio::test]
    async fn test_pending_queue_persistence() {
        let tmp = tempdir().unwrap();
        let p = Persistence::new(tmp.path().into());

        let e1 = sample_event();
        let e2 = sample_event();
        p.save_pending(&[e1.clone(), e2.clone()]).await.unwrap();

        let loaded = p.load_pending().await.unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, e1.id);
        assert_eq!(loaded[1].id, e2.id);
    }

    #[tokio::test]
    async fn test_pending_overwrites_previous() {
        let tmp = tempdir().unwrap();
        let p = Persistence::new(tmp.path().into());

        p.save_pending(&[sample_event()]).await.unwrap();
        let new_events = vec![sample_event(), sample_event(), sample_event()];
        p.save_pending(&new_events).await.unwrap();

        let loaded = p.load_pending().await.unwrap();
        assert_eq!(loaded.len(), 3);
    }

    #[tokio::test]
    async fn test_pending_clear() {
        let tmp = tempdir().unwrap();
        let p = Persistence::new(tmp.path().into());

        p.save_pending(&[sample_event()]).await.unwrap();
        p.save_pending(&[]).await.unwrap();

        let loaded = p.load_pending().await.unwrap();
        assert!(loaded.is_empty());
    }

    // ── Atomicity: file survives concurrent-like reads ──────────────────

    #[tokio::test]
    async fn test_atomic_write_creates_parent_dirs() {
        let tmp = tempdir().unwrap();
        let nested = tmp.path().join("a").join("b");
        let p = Persistence::new(nested);
        let session = sample_session();

        p.save_active_session(Some(&session)).await.unwrap();
        assert!(p.load_active_session().await.unwrap().is_some());
    }
}
