#![allow(dead_code)]

//! Join-code authentication middleware for the classroom HTTP server.
//!
//! When a teacher starts hosting, a random 4-digit code is generated.
//! Students must include this code in the `X-Lumin-Join-Code` header
//! on every classroom request. Missing or incorrect codes receive 401.

#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use tokio::sync::RwLock;

// ── State ────────────────────────────────────────────────────────────────────

/// The active join code for the current session.
/// `None` until the teacher generates one.
pub struct JoinCodeState {
    pub code: RwLock<Option<String>>,
}

impl Default for JoinCodeState {
    fn default() -> Self {
        Self {
            code: RwLock::new(None),
        }
    }
}

impl JoinCodeState {
    pub fn new(initial: String) -> Self {
        Self {
            code: RwLock::new(Some(initial)),
        }
    }
}

// ── Code generation ──────────────────────────────────────────────────────────

/// Generate a random 4-digit code (zero-padded, range 0000–9999).
pub fn generate_join_code() -> String {
    use rand::RngExt;
    let mut rng = rand::rng();
    format!("{:04}", rng.random_range(0..10000))
}

// ── Middleware ────────────────────────────────────────────────────────────────

/// Axum middleware that validates the `X-Lumin-Join-Code` header.
///
/// Returns `401 UNAUTHORIZED` if the header is missing, malformed, or incorrect.
/// Returns `401` if no code has been generated yet (teacher hasn't started).
#[cfg(test)]
pub async fn join_code_middleware(
    State(state): State<Arc<JoinCodeState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let header_code = req
        .headers()
        .get("X-Lumin-Join-Code")
        .and_then(|v| v.to_str().ok());

    let expected = state.code.read().await;

    match (header_code, expected.as_ref()) {
        (Some(c), Some(e)) if c == e => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_generate_join_code_uniqueness_is_likely() {
        // With 10000 possible codes and 50 generated, collision is astronomically unlikely.
        let codes: std::collections::HashSet<String> =
            (0..50).map(|_| generate_join_code()).collect();
        assert!(
            codes.len() >= 45,
            "expected at least 45 unique codes out of 50, got {}",
            codes.len()
        );
    }

    #[test]
    fn test_join_code_state_default_is_none() {
        let state = JoinCodeState::default();
        assert!(state.code.try_read().unwrap().is_none());
    }

    #[test]
    fn test_join_code_state_new() {
        let state = JoinCodeState::new("1234".into());
        assert_eq!(*state.code.try_read().unwrap().as_ref().unwrap(), "1234");
    }
}
