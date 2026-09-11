#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// QuizQuestion
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizQuestion {
    pub id: String,
    pub prompt: String,
    pub concept: String,
    pub accepted_answers: Vec<String>,
    #[serde(default)]
    pub misconception_answers: HashMap<String, String>,
    pub generic_misconception: String,
    #[serde(default)]
    pub hints: Vec<String>,
    pub explanation: String,
}

// ---------------------------------------------------------------------------
// Quiz
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quiz {
    pub id: String,
    pub title: String,
    pub subject: String,
    pub topic: Option<String>,
    #[serde(default)]
    pub questions: Vec<QuizQuestion>,
}

// ---------------------------------------------------------------------------
// LearningSession
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningSession {
    pub id: Uuid,
    pub quiz: Quiz,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub started_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// AnswerAnalysis (not Codable in Swift, but included for completeness)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerAnalysis {
    pub is_correct: bool,
    pub misconception: Option<String>,
}

// ---------------------------------------------------------------------------
// AnalysisEvent
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisEvent {
    pub id: Uuid,
    pub participant_token: String,
    pub session_id: Option<Uuid>,
    #[serde(rename = "questionID")]
    pub question_id: String,
    pub concept: String,
    pub misconception: Option<String>,
    pub correct: bool,
    pub hint_count: i32,
    pub retry_success: bool,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub submitted_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// LessonPlan
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonPlan {
    pub focus: String,
    #[serde(default)]
    pub steps: Vec<String>,
    pub check_question: String,
    pub teacher_note: String,
}

// ---------------------------------------------------------------------------
// SessionArchive
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionArchive {
    pub id: Uuid,
    pub session: LearningSession,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub ended_at: DateTime<Utc>,
    #[serde(default)]
    pub events: Vec<AnalysisEvent>,
    pub adopted_plan: Option<LessonPlan>,
}

// ---------------------------------------------------------------------------
// MisconceptionSummary
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MisconceptionSummary {
    pub name: String,
    pub count: i32,
    pub share: f64,
}

impl MisconceptionSummary {
    /// The `id` field is derived from `name` (matches Swift `Identifiable`).
    pub fn id(&self) -> &str {
        &self.name
    }
}

// ---------------------------------------------------------------------------
// ClassSummary
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassSummary {
    pub participant_count: i32,
    pub response_count: i32,
    pub correct_rate: f64,
    pub retry_success_rate: f64,
    pub average_hints: f64,
    #[serde(default)]
    pub misconceptions: Vec<MisconceptionSummary>,
}

// ---------------------------------------------------------------------------
// SessionBroadcast — teacher broadcasts session + quiz to students
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionBroadcast {
    pub session_id: Uuid,
    pub quiz: Quiz,
    pub join_code: String,
}

// ---------------------------------------------------------------------------
// EndSession — teacher ends the active session
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndSession {
    pub session_id: Uuid,
}

// ---------------------------------------------------------------------------
// AnalysisEventAck — response when a student submits an AnalysisEvent
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisEventAck {
    pub event_id: Uuid,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub received_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// StudentInfo — metadata about a connected student (no quiz answers)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentInfo {
    pub participant_token: String,
    pub connected_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// StudentList — response for GET /students
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentList {
    pub students: Vec<StudentInfo>,
}

// ---------------------------------------------------------------------------
// PeerMessage — manual serde to match Swift Codable encoding
// ---------------------------------------------------------------------------
//
// Swift JSON format:
//   {"type":"session","session":{...}}
//   {"type":"sessionEnded","sessionID":"uuid"}
//   {"type":"quiz","quiz":{...}}
//   {"type":"analysis","analysis":{...}}
//   {"type":"acknowledgment","eventID":"uuid"}

#[derive(Debug, Clone)]
pub enum PeerMessage {
    Session(LearningSession),
    SessionEnded(Uuid),
    Quiz(Quiz),
    Analysis(AnalysisEvent),
    Acknowledgment(Uuid),
}

impl Serialize for PeerMessage {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;

        let (tag, value) = match self {
            PeerMessage::Session(s) => (
                "session",
                serde_json::to_value(s).map_err(serde::ser::Error::custom)?,
            ),
            PeerMessage::SessionEnded(id) => (
                "sessionEnded",
                serde_json::to_value(id).map_err(serde::ser::Error::custom)?,
            ),
            PeerMessage::Quiz(q) => (
                "quiz",
                serde_json::to_value(q).map_err(serde::ser::Error::custom)?,
            ),
            PeerMessage::Analysis(a) => (
                "analysis",
                serde_json::to_value(a).map_err(serde::ser::Error::custom)?,
            ),
            PeerMessage::Acknowledgment(id) => (
                "acknowledgment",
                serde_json::to_value(id).map_err(serde::ser::Error::custom)?,
            ),
        };

        let key = match tag {
            "sessionEnded" => "sessionID",
            "acknowledgment" => "eventID",
            other => other,
        };

        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type", tag)?;
        map.serialize_entry(key, &value)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for PeerMessage {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;

        let msg_type = value
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| serde::de::Error::custom("missing \"type\" field"))?;

        match msg_type {
            "session" => {
                let session: LearningSession = serde_json::from_value(
                    value
                        .get("session")
                        .cloned()
                        .ok_or_else(|| serde::de::Error::custom("missing \"session\""))?,
                )
                .map_err(serde::de::Error::custom)?;
                Ok(PeerMessage::Session(session))
            }
            "sessionEnded" => {
                let id: Uuid = serde_json::from_value(
                    value
                        .get("sessionID")
                        .cloned()
                        .ok_or_else(|| serde::de::Error::custom("missing \"sessionID\""))?,
                )
                .map_err(serde::de::Error::custom)?;
                Ok(PeerMessage::SessionEnded(id))
            }
            "quiz" => {
                let quiz: Quiz = serde_json::from_value(
                    value
                        .get("quiz")
                        .cloned()
                        .ok_or_else(|| serde::de::Error::custom("missing \"quiz\""))?,
                )
                .map_err(serde::de::Error::custom)?;
                Ok(PeerMessage::Quiz(quiz))
            }
            "analysis" => {
                let analysis: AnalysisEvent = serde_json::from_value(
                    value
                        .get("analysis")
                        .cloned()
                        .ok_or_else(|| serde::de::Error::custom("missing \"analysis\""))?,
                )
                .map_err(serde::de::Error::custom)?;
                Ok(PeerMessage::Analysis(analysis))
            }
            "acknowledgment" => {
                let id: Uuid = serde_json::from_value(
                    value
                        .get("eventID")
                        .cloned()
                        .ok_or_else(|| serde::de::Error::custom("missing \"eventID\""))?,
                )
                .map_err(serde::de::Error::custom)?;
                Ok(PeerMessage::Acknowledgment(id))
            }
            other => Err(serde::de::Error::custom(format!(
                "unknown PeerMessage type: {other}"
            ))),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_question() -> QuizQuestion {
        let mut misconceptions = HashMap::new();
        misconceptions.insert("1/2".into(), "added numerators".into());
        QuizQuestion {
            id: "q1".into(),
            prompt: "What is 1/2 + 1/3?".into(),
            concept: "fraction addition".into(),
            accepted_answers: vec!["5/6".into()],
            misconception_answers: misconceptions,
            generic_misconception: "incorrect fraction addition".into(),
            hints: vec!["Find a common denominator".into()],
            explanation: "Convert to 3/6 + 2/6 = 5/6".into(),
        }
    }

    fn sample_quiz() -> Quiz {
        Quiz {
            id: "quiz1".into(),
            title: "Fractions Quiz".into(),
            subject: "Math".into(),
            topic: Some("Fractions".into()),
            questions: vec![sample_question()],
        }
    }

    #[test]
    fn roundtrip_quiz_question() {
        let q = sample_question();
        let json = serde_json::to_string(&q).unwrap();
        let back: QuizQuestion = serde_json::from_str(&json).unwrap();
        assert_eq!(q.id, back.id);
        assert_eq!(q.prompt, back.prompt);
        assert_eq!(q.concept, back.concept);
        assert_eq!(q.accepted_answers, back.accepted_answers);
        assert_eq!(q.misconception_answers, back.misconception_answers);
        assert_eq!(q.hints, back.hints);
    }

    #[test]
    fn roundtrip_quiz() {
        let quiz = sample_quiz();
        let json = serde_json::to_string(&quiz).unwrap();
        let back: Quiz = serde_json::from_str(&json).unwrap();
        assert_eq!(quiz.id, back.id);
        assert_eq!(quiz.title, back.title);
        assert_eq!(quiz.topic, back.topic);
        assert_eq!(quiz.questions.len(), back.questions.len());
    }

    #[test]
    fn roundtrip_learning_session() {
        let session = LearningSession {
            id: Uuid::new_v4(),
            quiz: sample_quiz(),
            started_at: Utc::now(),
        };
        let json = serde_json::to_string(&session).unwrap();
        let back: LearningSession = serde_json::from_str(&json).unwrap();
        assert_eq!(session.id, back.id);
        assert_eq!(session.quiz.id, back.quiz.id);
    }

    #[test]
    fn roundtrip_answer_analysis() {
        let aa = AnswerAnalysis {
            is_correct: true,
            misconception: None,
        };
        let json = serde_json::to_string(&aa).unwrap();
        let back: AnswerAnalysis = serde_json::from_str(&json).unwrap();
        assert_eq!(aa.is_correct, back.is_correct);
        assert_eq!(aa.misconception, back.misconception);
    }

    #[test]
    fn roundtrip_analysis_event() {
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
        let json = serde_json::to_string(&event).unwrap();
        let back: AnalysisEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event.id, back.id);
        assert_eq!(event.participant_token, back.participant_token);
        assert_eq!(event.question_id, back.question_id);
    }

    #[test]
    fn roundtrip_lesson_plan() {
        let plan = LessonPlan {
            focus: "Fractions".into(),
            steps: vec!["Step 1".into()],
            check_question: "What is 1/2 + 1/3?".into(),
            teacher_note: "Watch for common errors".into(),
        };
        let json = serde_json::to_string(&plan).unwrap();
        let back: LessonPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan.focus, back.focus);
        assert_eq!(plan.steps, back.steps);
    }

    #[test]
    fn roundtrip_session_archive() {
        let archive = SessionArchive {
            id: Uuid::new_v4(),
            session: LearningSession {
                id: Uuid::new_v4(),
                quiz: sample_quiz(),
                started_at: Utc::now(),
            },
            ended_at: Utc::now(),
            events: vec![],
            adopted_plan: None,
        };
        let json = serde_json::to_string(&archive).unwrap();
        let back: SessionArchive = serde_json::from_str(&json).unwrap();
        assert_eq!(archive.id, back.id);
    }

    #[test]
    fn roundtrip_misconception_summary() {
        let ms = MisconceptionSummary {
            name: "added numerators".into(),
            count: 5,
            share: 0.25,
        };
        let json = serde_json::to_string(&ms).unwrap();
        let back: MisconceptionSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(ms.name, back.name);
        assert_eq!(ms.count, back.count);
        assert!((ms.share - back.share).abs() < f64::EPSILON);
    }

    #[test]
    fn roundtrip_class_summary() {
        let cs = ClassSummary {
            participant_count: 30,
            response_count: 28,
            correct_rate: 0.75,
            retry_success_rate: 0.6,
            average_hints: 1.2,
            misconceptions: vec![MisconceptionSummary {
                name: "added numerators".into(),
                count: 5,
                share: 0.25,
            }],
        };
        let json = serde_json::to_string(&cs).unwrap();
        let back: ClassSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(cs.participant_count, back.participant_count);
        assert_eq!(cs.misconceptions.len(), 1);
    }

    // --- PeerMessage roundtrip tests ---

    #[test]
    fn roundtrip_peer_message_session() {
        let msg = PeerMessage::Session(LearningSession {
            id: Uuid::new_v4(),
            quiz: sample_quiz(),
            started_at: Utc::now(),
        });
        let json = serde_json::to_string(&msg).unwrap();
        let back: PeerMessage = serde_json::from_str(&json).unwrap();
        match back {
            PeerMessage::Session(s) => assert_eq!(s.quiz.id, "quiz1"),
            _ => panic!("expected Session variant"),
        }
    }

    #[test]
    fn roundtrip_peer_message_session_ended() {
        let id = Uuid::new_v4();
        let msg = PeerMessage::SessionEnded(id);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"sessionEnded\""));
        assert!(json.contains("\"sessionID\""));
        let back: PeerMessage = serde_json::from_str(&json).unwrap();
        match back {
            PeerMessage::SessionEnded(back_id) => assert_eq!(back_id, id),
            _ => panic!("expected SessionEnded variant"),
        }
    }

    #[test]
    fn roundtrip_peer_message_quiz() {
        let msg = PeerMessage::Quiz(sample_quiz());
        let json = serde_json::to_string(&msg).unwrap();
        let back: PeerMessage = serde_json::from_str(&json).unwrap();
        match back {
            PeerMessage::Quiz(q) => assert_eq!(q.title, "Fractions Quiz"),
            _ => panic!("expected Quiz variant"),
        }
    }

    #[test]
    fn roundtrip_peer_message_analysis() {
        let event = AnalysisEvent {
            id: Uuid::new_v4(),
            participant_token: "tok".into(),
            session_id: None,
            question_id: "q1".into(),
            concept: "fractions".into(),
            misconception: None,
            correct: true,
            hint_count: 0,
            retry_success: false,
            submitted_at: Utc::now(),
        };
        let msg = PeerMessage::Analysis(event);
        let json = serde_json::to_string(&msg).unwrap();
        let back: PeerMessage = serde_json::from_str(&json).unwrap();
        match back {
            PeerMessage::Analysis(a) => assert_eq!(a.question_id, "q1"),
            _ => panic!("expected Analysis variant"),
        }
    }

    #[test]
    fn roundtrip_peer_message_acknowledgment() {
        let id = Uuid::new_v4();
        let msg = PeerMessage::Acknowledgment(id);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"acknowledgment\""));
        assert!(json.contains("\"eventID\""));
        let back: PeerMessage = serde_json::from_str(&json).unwrap();
        match back {
            PeerMessage::Acknowledgment(back_id) => assert_eq!(back_id, id),
            _ => panic!("expected Acknowledgment variant"),
        }
    }
}
