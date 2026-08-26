#![allow(dead_code)]

//! Reliable delivery primitives for AnalysisEvent transport.
//!
//! - **Student-side** ([`PendingQueue`]): persists unsent events to
//!   `app_data_dir/pending_analysis.json` and flushes oldest-first on connect.
//!   Events are removed only after receiving an ACK from the teacher.
//!
//! - **Teacher-side** ([`EventDeduplicator`]): deduplicates incoming events by
//!   their UUID and persists the set of received IDs to
//!   `app_data_dir/received_events.json`.

use crate::lumin_core::models::AnalysisEvent;
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::fs;

// ---------------------------------------------------------------------------
// PendingQueue — student-side
// ---------------------------------------------------------------------------

/// Persistent queue of unsent analysis events.
///
/// Events are stored oldest-first in `app_data_dir/pending_analysis.json`.
/// On network connect, the caller should flush via [`PendingQueue::load`] and
/// send each event, then call [`PendingQueue::acknowledge`] upon receiving
/// the teacher's ACK.
#[derive(Debug, Default)]
pub struct PendingQueue {
    path: PathBuf,
}

impl PendingQueue {
    /// Create a new queue rooted at `app_dir`.
    pub fn new(app_dir: PathBuf) -> Self {
        Self {
            path: app_dir.join("pending_analysis.json"),
        }
    }

    /// Append an event to the pending queue and persist to disk.
    ///
    /// Idempotent: duplicate events (same `id`) are silently ignored.
    pub async fn push(&self, event: &AnalysisEvent) -> Result<(), String> {
        let mut events = self.load().await.unwrap_or_default();
        if events.iter().any(|e| e.id == event.id) {
            return Ok(());
        }
        events.push(event.clone());
        self.save(&events).await
    }

    /// Remove an event by ID after receiving its ACK.
    ///
    /// No-op if the event is not in the queue.
    pub async fn acknowledge(&self, event_id: uuid::Uuid) -> Result<(), String> {
        let mut events = self.load().await.unwrap_or_default();
        let before = events.len();
        events.retain(|e| e.id != event_id);
        if events.len() == before {
            return Ok(());
        }
        self.save(&events).await
    }

    /// Load all pending events, oldest first (insertion order).
    pub async fn load(&self) -> Result<Vec<AnalysisEvent>, String> {
        match fs::read(&self.path).await {
            Ok(data) => serde_json::from_slice(&data)
                .map_err(|e| format!("failed to parse pending queue: {e}")),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(format!("failed to read pending queue: {e}")),
        }
    }

    /// Save the queue to disk atomically (write-to-temp, then rename).
    async fn save(&self, events: &[AnalysisEvent]) -> Result<(), String> {
        let tmp = self.path.with_extension("json.tmp");
        let data = serde_json::to_vec_pretty(events)
            .map_err(|e| format!("failed to serialize pending queue: {e}"))?;
        fs::write(&tmp, &data)
            .await
            .map_err(|e| format!("failed to write pending queue tmp: {e}"))?;
        fs::rename(&tmp, &self.path)
            .await
            .map_err(|e| format!("failed to rename pending queue: {e}"))
    }
}

// ---------------------------------------------------------------------------
// EventDeduplicator — teacher-side
// ---------------------------------------------------------------------------

/// Deduplicates incoming events by their UUID.
///
/// Persists received event IDs to `app_data_dir/received_events.json` so that
/// duplicates survive process restarts.
#[derive(Debug, Default)]
pub struct EventDeduplicator {
    received: HashSet<uuid::Uuid>,
    path: PathBuf,
}

impl EventDeduplicator {
    /// Create a new deduplicator rooted at `app_dir`.
    pub fn new(app_dir: PathBuf) -> Self {
        Self {
            received: HashSet::new(),
            path: app_dir.join("received_events.json"),
        }
    }

    /// Returns `true` if the event has already been seen (duplicate).
    pub fn is_duplicate(&self, event: &AnalysisEvent) -> bool {
        self.received.contains(&event.id)
    }

    /// Mark an event as received. Call after processing.
    pub fn mark_received(&mut self, event: &AnalysisEvent) {
        self.received.insert(event.id);
    }

    /// Persist the set of received IDs to disk atomically.
    pub async fn save(&self) -> Result<(), String> {
        let ids: Vec<String> = self.received.iter().map(|id| id.to_string()).collect();
        let tmp = self.path.with_extension("json.tmp");
        let data = serde_json::to_vec_pretty(&ids)
            .map_err(|e| format!("failed to serialize received events: {e}"))?;
        fs::write(&tmp, &data)
            .await
            .map_err(|e| format!("failed to write received events tmp: {e}"))?;
        fs::rename(&tmp, &self.path)
            .await
            .map_err(|e| format!("failed to rename received events: {e}"))
    }

    /// Load previously received IDs from disk (call on startup).
    pub async fn load(&mut self) -> Result<(), String> {
        match fs::read(&self.path).await {
            Ok(data) => {
                let ids: Vec<String> = serde_json::from_slice(&data)
                    .map_err(|e| format!("failed to parse received events: {e}"))?;
                self.received = ids
                    .iter()
                    .filter_map(|s| uuid::Uuid::parse_str(s).ok())
                    .collect();
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(format!("failed to read received events: {e}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lumin_core::models::AnalysisEvent;
    use chrono::Utc;

    fn make_event(id_str: &str) -> AnalysisEvent {
        AnalysisEvent {
            id: uuid::Uuid::parse_str(id_str).unwrap_or_else(|_| uuid::Uuid::new_v4()),
            participant_token: "tok_test".into(),
            session_id: None,
            question_id: "q1".into(),
            concept: "fractions".into(),
            misconception: None,
            correct: true,
            hint_count: 0,
            retry_success: false,
            submitted_at: Utc::now(),
        }
    }

    // ── PendingQueue ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_pending_queue_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let q = PendingQueue::new(tmp.path().into());
        let event = make_event("550e8400-e29b-41d4-a716-446655440000");
        q.push(&event).await.unwrap();
        let loaded = q.load().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, event.id);
        q.acknowledge(event.id).await.unwrap();
        assert_eq!(q.load().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_pending_queue_preserves_order() {
        let tmp = tempfile::tempdir().unwrap();
        let q = PendingQueue::new(tmp.path().into());
        let e1 = make_event("aaaaaaaa-aaaa-4aaa-aaaa-aaaaaaaaaaaa");
        let e2 = make_event("bbbbbbbb-bbbb-4bbb-bbbb-bbbbbbbbbbbb");
        let e3 = make_event("cccccccc-cccc-4ccc-cccc-cccccccccccc");
        q.push(&e1).await.unwrap();
        q.push(&e2).await.unwrap();
        q.push(&e3).await.unwrap();
        let loaded = q.load().await.unwrap();
        assert_eq!(loaded[0].id, e1.id);
        assert_eq!(loaded[1].id, e2.id);
        assert_eq!(loaded[2].id, e3.id);
    }

    #[tokio::test]
    async fn test_pending_queue_idempotent_push() {
        let tmp = tempfile::tempdir().unwrap();
        let q = PendingQueue::new(tmp.path().into());
        let event = make_event("550e8400-e29b-41d4-a716-446655440000");
        q.push(&event).await.unwrap();
        q.push(&event).await.unwrap();
        assert_eq!(q.load().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_pending_queue_empty_on_missing_file() {
        let tmp = tempfile::tempdir().unwrap();
        let q = PendingQueue::new(tmp.path().into());
        let loaded = q.load().await.unwrap();
        assert!(loaded.is_empty());
    }

    #[tokio::test]
    async fn test_pending_queue_ack_nonexistent_is_noop() {
        let tmp = tempfile::tempdir().unwrap();
        let q = PendingQueue::new(tmp.path().into());
        let event = make_event("550e8400-e29b-41d4-a716-446655440000");
        q.push(&event).await.unwrap();
        let other = make_event("660e8400-e29b-41d4-a716-446655440001");
        q.acknowledge(other.id).await.unwrap();
        assert_eq!(q.load().await.unwrap().len(), 1);
    }

    // ── EventDeduplicator ────────────────────────────────────────────────

    #[tokio::test]
    async fn test_deduplicator_rejects_duplicate() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dedup = EventDeduplicator::new(tmp.path().into());
        let event = make_event("550e8400-e29b-41d4-a716-446655440000");
        assert!(!dedup.is_duplicate(&event));
        dedup.mark_received(&event);
        assert!(dedup.is_duplicate(&event));
    }

    #[tokio::test]
    async fn test_deduplicator_persistence() {
        let tmp = tempfile::tempdir().unwrap();
        let event = make_event("550e8400-e29b-41d4-a716-446655440000");
        {
            let mut dedup = EventDeduplicator::new(tmp.path().into());
            dedup.mark_received(&event);
            dedup.save().await.unwrap();
        }
        {
            let mut dedup = EventDeduplicator::new(tmp.path().into());
            dedup.load().await.unwrap();
            assert!(dedup.is_duplicate(&event));
        }
    }

    #[tokio::test]
    async fn test_deduplicator_empty_load() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dedup = EventDeduplicator::new(tmp.path().into());
        dedup.load().await.unwrap();
        assert!(dedup.received.is_empty());
    }

    #[tokio::test]
    async fn test_deduplicator_multiple_events() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dedup = EventDeduplicator::new(tmp.path().into());
        let e1 = make_event("aaaaaaaa-aaaa-4aaa-aaaa-aaaaaaaaaaaa");
        let e2 = make_event("bbbbbbbb-bbbb-4bbb-bbbb-bbbbbbbbbbbb");
        assert!(!dedup.is_duplicate(&e1));
        assert!(!dedup.is_duplicate(&e2));
        dedup.mark_received(&e1);
        dedup.mark_received(&e2);
        assert!(dedup.is_duplicate(&e1));
        assert!(dedup.is_duplicate(&e2));
        // Save and reload
        dedup.save().await.unwrap();
        let mut dedup2 = EventDeduplicator::new(tmp.path().into());
        dedup2.load().await.unwrap();
        assert!(dedup2.is_duplicate(&e1));
        assert!(dedup2.is_duplicate(&e2));
    }
}
