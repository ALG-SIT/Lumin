//! Encrypted TCP transport for classroom traffic.
use std::{
    collections::{HashMap, VecDeque},
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use snow::{params::NoiseParams, Builder, TransportState};
use tauri::Emitter;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{watch, Mutex, Semaphore},
};
use uuid::Uuid;

use crate::{
    lumin_core::models::{AnalysisEvent, AnalysisEventAck, Quiz, StudentInfo},
    network::server::ServerState,
};

const PATTERN: &str = "Noise_XX_25519_ChaChaPoly_SHA256";
const MAX_RECORD: usize = 65_535;
const MAX_JSON: usize = 1_048_576;
const FRAGMENT_SIZE: usize = 60_000;
const IO_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Request {
    Prepare,
    Join { session_id: Uuid, join_code: String },
    Analysis(AnalysisEvent),
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Response {
    Prepared {
        session_id: Uuid,
    },
    Joined {
        session_id: Uuid,
        participant_token: String,
        quiz: Quiz,
    },
    Ack(AnalysisEventAck),
    Pong,
    Error {
        message: String,
    },
}

pub struct Connection<S> {
    stream: S,
    cipher: TransportState,
    pub remote_static: Vec<u8>,
}

#[derive(Clone)]
struct PeerContext {
    private_key: Arc<Vec<u8>>,
    state: Arc<ServerState>,
    students: Arc<Mutex<HashMap<Vec<u8>, String>>>,
    failures: Arc<Mutex<HashMap<IpAddr, VecDeque<Instant>>>>,
    handshakes: Arc<Semaphore>,
}

impl<S: AsyncRead + AsyncWrite + Unpin> Connection<S> {
    pub async fn handshake_client(mut stream: S, local_key: &[u8]) -> Result<Self, String> {
        let mut noise = Builder::new(params()?)
            .local_private_key(local_key)
            .map_err(err)?
            .build_initiator()
            .map_err(err)?;
        let mut out = vec![0u8; MAX_RECORD];
        let mut input = vec![0u8; MAX_RECORD];
        let n = noise.write_message(b"LUMIN\x02", &mut out).map_err(err)?;
        write_frame(&mut stream, &out[..n]).await?;
        let n = read_frame(&mut stream, &mut input).await?;
        let plain = noise.read_message(&input[..n], &mut out).map_err(err)?;
        if &out[..plain] != b"LUMIN\x02" {
            return Err("非対応プロトコルです。教師・生徒のアプリを更新してください".into());
        }
        let n = noise.write_message(b"LUMIN\x02", &mut input).map_err(err)?;
        write_frame(&mut stream, &input[..n]).await?;
        let remote_static = noise
            .get_remote_static()
            .ok_or("教師の鍵を確認できません")?
            .to_vec();
        let cipher = noise.into_transport_mode().map_err(err)?;
        Ok(Self {
            stream,
            cipher,
            remote_static,
        })
    }

    pub async fn handshake_server(mut stream: S, key: &[u8]) -> Result<Self, String> {
        let mut noise = Builder::new(params()?)
            .local_private_key(key)
            .map_err(err)?
            .build_responder()
            .map_err(err)?;
        let mut input = vec![0u8; MAX_RECORD];
        let mut out = vec![0u8; MAX_RECORD];
        let n = read_frame(&mut stream, &mut input).await?;
        let plain = noise.read_message(&input[..n], &mut out).map_err(err)?;
        if &out[..plain] != b"LUMIN\x02" {
            return Err("非対応プロトコルです".into());
        }
        let n = noise.write_message(b"LUMIN\x02", &mut out).map_err(err)?;
        write_frame(&mut stream, &out[..n]).await?;
        let n = read_frame(&mut stream, &mut input).await?;
        let plain = noise.read_message(&input[..n], &mut out).map_err(err)?;
        if &out[..plain] != b"LUMIN\x02" {
            return Err("非対応プロトコルです".into());
        }
        let remote_static = noise
            .get_remote_static()
            .ok_or("生徒の鍵を確認できません")?
            .to_vec();
        let cipher = noise.into_transport_mode().map_err(err)?;
        Ok(Self {
            stream,
            cipher,
            remote_static,
        })
    }

    pub async fn send(&mut self, value: &impl Serialize) -> Result<(), String> {
        let plain = serde_json::to_vec(value).map_err(err)?;
        if plain.len() > MAX_JSON {
            return Err("通信データが上限を超えています".into());
        }
        let message_id = *Uuid::new_v4().as_bytes();
        let fragments = plain.chunks(FRAGMENT_SIZE).collect::<Vec<_>>();
        let count = u16::try_from(fragments.len()).map_err(err)?;
        let mut out = vec![0u8; MAX_RECORD];
        for (index, fragment) in fragments.into_iter().enumerate() {
            let mut chunk = Vec::with_capacity(20 + fragment.len());
            chunk.extend_from_slice(&message_id);
            chunk.extend_from_slice(&count.to_be_bytes());
            chunk.extend_from_slice(&(index as u16).to_be_bytes());
            chunk.extend_from_slice(fragment);
            let n = self.cipher.write_message(&chunk, &mut out).map_err(err)?;
            write_frame(&mut self.stream, &out[..n]).await?;
        }
        Ok(())
    }

    pub async fn receive<T: for<'de> Deserialize<'de>>(&mut self) -> Result<T, String> {
        let mut input = vec![0u8; MAX_RECORD];
        let mut out = vec![0u8; MAX_RECORD];
        let mut message_id = None;
        let mut parts: Vec<Option<Vec<u8>>> = Vec::new();
        let mut total = 0usize;
        loop {
            let n = read_frame(&mut self.stream, &mut input).await?;
            let len = self
                .cipher
                .read_message(&input[..n], &mut out)
                .map_err(err)?;
            if len < 20 {
                return Err("通信レコードの形式が不正です".into());
            }
            let id: [u8; 16] = out[..16].try_into().map_err(err)?;
            let count = u16::from_be_bytes([out[16], out[17]]) as usize;
            let index = u16::from_be_bytes([out[18], out[19]]) as usize;
            if count == 0
                || count > 18
                || index >= count
                || message_id.is_some_and(|expected| expected != id)
            {
                return Err("通信断片の順序が不正です".into());
            }
            if message_id.is_none() {
                message_id = Some(id);
                parts = vec![None; count];
            }
            if parts.len() != count || parts[index].is_some() {
                return Err("通信断片が重複しています".into());
            }
            let part = out[20..len].to_vec();
            total += part.len();
            if total > MAX_JSON {
                return Err("通信データが上限を超えています".into());
            }
            parts[index] = Some(part);
            if parts.iter().all(Option::is_some) {
                let mut plain = Vec::with_capacity(total);
                for part in parts {
                    plain.extend(part.ok_or("通信断片が欠けています")?);
                }
                return serde_json::from_slice(&plain).map_err(err);
            }
        }
    }
}

pub fn keypair() -> Result<snow::Keypair, String> {
    Builder::new(params()?).generate_keypair().map_err(err)
}

pub fn fingerprint(key: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(&Sha256::digest(key)[..16])
        .as_bytes()
        .chunks(4)
        .map(|c| std::str::from_utf8(c).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn start_heartbeat(connection: &Arc<Mutex<Connection<TcpStream>>>) -> watch::Sender<bool> {
    let (stop_tx, mut stop_rx) = watch::channel(false);
    let connection = Arc::downgrade(connection);
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = stop_rx.changed() => break,
                _ = tokio::time::sleep(Duration::from_secs(15)) => {
                    let Some(connection) = connection.upgrade() else {
                        break;
                    };
                    let mut connection = connection.lock().await;
                    if connection.send(&Request::Ping).await.is_err()
                        || !matches!(connection.receive::<Response>().await, Ok(Response::Pong))
                    {
                        break;
                    }
                }
            }
        }
    });
    stop_tx
}

fn params() -> Result<NoiseParams, String> {
    PATTERN.parse().map_err(err)
}
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

async fn read_frame<S: AsyncRead + Unpin>(stream: &mut S, buf: &mut [u8]) -> Result<usize, String> {
    tokio::time::timeout(IO_TIMEOUT, async {
        let len = stream.read_u16().await? as usize;
        if len == 0 || len > buf.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid frame length",
            ));
        }
        stream.read_exact(&mut buf[..len]).await?;
        Ok::<usize, std::io::Error>(len)
    })
    .await
    .map_err(|_| "通信がタイムアウトしました".to_string())?
    .map_err(err)
}

async fn write_frame<S: AsyncWrite + Unpin>(stream: &mut S, data: &[u8]) -> Result<(), String> {
    if data.is_empty() || data.len() > MAX_RECORD {
        return Err("通信レコードが上限を超えています".into());
    }
    tokio::time::timeout(IO_TIMEOUT, async {
        stream.write_u16(data.len() as u16).await?;
        stream.write_all(data).await?;
        stream.flush().await
    })
    .await
    .map_err(|_| "通信がタイムアウトしました".to_string())?
    .map_err(err)
}

pub async fn serve(
    port: u16,
    state: Arc<ServerState>,
    shutdown: watch::Receiver<bool>,
) -> Result<(tokio::task::JoinHandle<()>, String), String> {
    let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port)))
        .await
        .map_err(err)?;
    serve_with_listener(listener, state, shutdown)
}

pub(crate) fn serve_with_listener(
    listener: TcpListener,
    state: Arc<ServerState>,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(tokio::task::JoinHandle<()>, String), String> {
    let key = keypair()?;
    let print = fingerprint(&key.public);
    let context = PeerContext {
        private_key: Arc::new(key.private),
        state,
        students: Arc::new(Mutex::new(HashMap::<Vec<u8>, String>::new())),
        failures: Arc::new(Mutex::new(HashMap::<IpAddr, VecDeque<Instant>>::new())),
        handshakes: Arc::new(Semaphore::new(16)),
    };
    let slots = Arc::new(Semaphore::new(128));
    let task = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = shutdown.changed() => break,
                accepted = listener.accept() => {
                    let Ok((socket, peer)) = accepted else { continue; };
                    let Ok(slot) = slots.clone().try_acquire_owned() else { continue; };
                    let context = context.clone();
                    let stop=shutdown.clone();
                    tokio::spawn(async move { let _slot=slot; let _=serve_peer(socket,peer.ip(),context,stop).await; });
                }
            }
        }
    });
    Ok((task, print))
}

async fn serve_peer(
    socket: TcpStream,
    peer_ip: IpAddr,
    context: PeerContext,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(), String> {
    let _ = socket.set_nodelay(true);
    let _handshake = context
        .handshakes
        .clone()
        .acquire_owned()
        .await
        .map_err(err)?;
    let mut conn = tokio::time::timeout(
        Duration::from_secs(5),
        Connection::handshake_server(socket, &context.private_key),
    )
    .await
    .map_err(|_| "Noise ハンドシェイクがタイムアウトしました".to_string())??;
    drop(_handshake);
    let remote = conn.remote_static.clone();
    let state = &context.state;
    let peers = &context.students;
    let failures = &context.failures;
    loop {
        let req: Request = tokio::select! {
            _ = shutdown.changed() => return Ok(()),
            result = conn.receive() => result?,
        };
        let response = match req {
            Request::Prepare => {
                let session = state
                    .active_session
                    .read()
                    .await
                    .ok_or("授業は終了しました")?;
                Response::Prepared {
                    session_id: session,
                }
            }
            Request::Join {
                session_id,
                join_code,
            } => {
                let active = *state.active_session.read().await;
                let code = state.join_code.code.read().await.clone();
                let is_match =
                    active == Some(session_id) && code.as_deref() == Some(join_code.as_str());

                let mut failures_lock = failures.lock().await;
                let now = Instant::now();
                failures_lock.retain(|_, attempts| {
                    while attempts
                        .front()
                        .is_some_and(|at| now.duration_since(*at) >= Duration::from_secs(60))
                    {
                        attempts.pop_front();
                    }
                    !attempts.is_empty()
                });
                let attempts = failures_lock.entry(peer_ip).or_default();
                let blocked = attempts.len() >= 20;

                if blocked || !is_match {
                    if !blocked {
                        attempts.push_back(now);
                    }
                    drop(failures_lock);
                    Response::Error {
                        message: if blocked {
                            "参加試行が多すぎます。1分後に再度お試しください"
                        } else {
                            "参加コードまたは授業情報が正しくありません"
                        }
                        .into(),
                    }
                } else {
                    failures_lock.remove(&peer_ip);
                    drop(failures_lock);

                    if let Some(quiz) = state.quiz.read().await.clone() {
                        let mut peers = peers.lock().await;
                        let token = peers
                            .entry(remote.clone())
                            .or_insert_with(|| Uuid::new_v4().to_string())
                            .clone();
                        let mut roster = state.students.write().await;
                        if !roster.iter().any(|s| s.participant_token == token) {
                            roster.push(StudentInfo {
                                participant_token: token.clone(),
                                connected_at: chrono::Utc::now(),
                            });
                        }
                        Response::Joined {
                            session_id,
                            participant_token: token,
                            quiz,
                        }
                    } else {
                        Response::Error {
                            message: "クイズが配信されていません".into(),
                        }
                    }
                }
            }
            Request::Analysis(mut event) => {
                let active_session = *state.active_session.read().await;
                let token = peers.lock().await.get(&remote).cloned();
                if active_session.is_none()
                    || event.session_id != active_session
                    || token.as_deref() != Some(event.participant_token.as_str())
                    || !state
                        .students
                        .read()
                        .await
                        .iter()
                        .any(|s| s.participant_token == event.participant_token)
                {
                    Response::Error {
                        message: "参加者または授業を確認できません".into(),
                    }
                } else {
                    let mut events = state.analysis_events.lock().await;
                    if !events.iter().any(|e| e.id == event.id) {
                        event.session_id = active_session;
                        events.push(event.clone());
                        if let Some(app) = &state.event_app {
                            let _ = app.emit("analysis-event", &event);
                        }
                    }
                    Response::Ack(AnalysisEventAck {
                        event_id: event.id,
                        received_at: chrono::Utc::now(),
                    })
                }
            }
            Request::Ping => Response::Pong,
        };
        let close = matches!(response, Response::Error { .. });
        conn.send(&response).await?;
        if close {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_format() {
        let key = [0xabu8; 32];
        let fp = fingerprint(&key);
        let parts: Vec<&str> = fp.split('-').collect();
        assert_eq!(parts.len(), 8);
        for part in parts {
            assert_eq!(part.len(), 4);
            assert!(part.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[tokio::test]
    async fn test_handshake_client_server_and_ping_pong() {
        let client_keys = keypair().unwrap();
        let server_keys = keypair().unwrap();

        let (client_io, server_io) = tokio::io::duplex(65536);

        let server_task = tokio::spawn(async move {
            let mut conn = Connection::handshake_server(server_io, &server_keys.private)
                .await
                .unwrap();
            let req: Request = conn.receive().await.unwrap();
            assert!(matches!(req, Request::Ping));
            conn.send(&Response::Pong).await.unwrap();
            conn
        });

        let client_task = tokio::spawn(async move {
            let mut conn = Connection::handshake_client(client_io, &client_keys.private)
                .await
                .unwrap();
            conn.send(&Request::Ping).await.unwrap();
            let resp: Response = conn.receive().await.unwrap();
            assert!(matches!(resp, Response::Pong));
            conn
        });

        let (server_conn, client_conn) = tokio::try_join!(server_task, client_task).unwrap();
        assert_eq!(server_conn.remote_static, client_keys.public);
        assert_eq!(client_conn.remote_static, server_keys.public);
    }

    #[tokio::test]
    async fn test_large_payload_fragmentation_roundtrip() {
        let client_keys = keypair().unwrap();
        let server_keys = keypair().unwrap();
        let (client_io, server_io) = tokio::io::duplex(131072);

        #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
        struct LargePayload {
            data: String,
        }

        let large_string = "A".repeat(150_000); // 150KB -> exceeds 60_000 byte fragment size

        let server_task = tokio::spawn(async move {
            let mut conn = Connection::handshake_server(server_io, &server_keys.private)
                .await
                .unwrap();
            let payload: LargePayload = conn.receive().await.unwrap();
            conn.send(&payload).await.unwrap();
        });

        let client_task = tokio::spawn(async move {
            let mut conn = Connection::handshake_client(client_io, &client_keys.private)
                .await
                .unwrap();
            let payload = LargePayload {
                data: large_string.clone(),
            };
            conn.send(&payload).await.unwrap();
            let received: LargePayload = conn.receive().await.unwrap();
            assert_eq!(received.data, large_string);
        });

        tokio::try_join!(server_task, client_task).unwrap();
    }

    #[tokio::test]
    async fn test_invalid_handshake_payload_rejected() {
        let server_keys = keypair().unwrap();
        let (mut client_io, server_io) = tokio::io::duplex(65536);

        let server_task = tokio::spawn(async move {
            Connection::handshake_server(server_io, &server_keys.private).await
        });

        // Write an invalid frame from client
        let _ = write_frame(&mut client_io, b"INVALID_NOISE_INIT").await;

        let result = server_task.await.unwrap();
        assert!(result.is_err(), "Server should reject invalid handshake");
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    use crate::network::auth::JoinCodeState;
    use tokio::io::DuplexStream;

    async fn connected_pair() -> (Connection<DuplexStream>, Connection<DuplexStream>) {
        let client_keys = keypair().unwrap();
        let server_keys = keypair().unwrap();
        let (client_io, server_io) = tokio::io::duplex(4 * MAX_RECORD);
        let (client, server) = tokio::join!(
            Connection::handshake_client(client_io, &client_keys.private),
            Connection::handshake_server(server_io, &server_keys.private),
        );
        (client.unwrap(), server.unwrap())
    }

    /// 暗号化済みの生レコードを送る(断片ヘッダを任意に組み立てるため)。
    async fn send_raw_record(conn: &mut Connection<DuplexStream>, chunk: &[u8]) {
        let mut out = vec![0u8; MAX_RECORD];
        let n = conn.cipher.write_message(chunk, &mut out).unwrap();
        write_frame(&mut conn.stream, &out[..n]).await.unwrap();
    }

    fn fragment(id: [u8; 16], count: u16, index: u16, body: &[u8]) -> Vec<u8> {
        let mut chunk = Vec::with_capacity(20 + body.len());
        chunk.extend_from_slice(&id);
        chunk.extend_from_slice(&count.to_be_bytes());
        chunk.extend_from_slice(&index.to_be_bytes());
        chunk.extend_from_slice(body);
        chunk
    }

    fn test_quiz() -> Quiz {
        Quiz {
            id: "quiz-1".into(),
            title: "Test Quiz".into(),
            subject: "math".into(),
            topic: None,
            questions: vec![],
        }
    }

    fn test_state(active: Option<Uuid>, code: &str, quiz: Option<Quiz>) -> Arc<ServerState> {
        Arc::new(ServerState {
            session_id: active.map(|id| id.to_string()),
            join_code: Arc::new(JoinCodeState::new(code.into())),
            teacher_token: "teacher-token".into(),
            active_session: tokio::sync::RwLock::new(active),
            students: tokio::sync::RwLock::new(Vec::new()),
            analysis_events: Arc::new(Mutex::new(Vec::new())),
            quiz: tokio::sync::RwLock::new(quiz),
            event_app: None,
        })
    }

    struct TestServer {
        port: u16,
        fingerprint: String,
        shutdown: watch::Sender<bool>,
        handle: tokio::task::JoinHandle<()>,
    }

    async fn start_server(state: Arc<ServerState>) -> TestServer {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let (shutdown, shutdown_rx) = watch::channel(false);
        let (handle, fingerprint) = serve_with_listener(listener, state, shutdown_rx).unwrap();
        TestServer {
            port,
            fingerprint,
            shutdown,
            handle,
        }
    }

    async fn connect(port: u16, key: &[u8]) -> Connection<TcpStream> {
        let stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        Connection::handshake_client(stream, key).await.unwrap()
    }

    async fn request(conn: &mut Connection<TcpStream>, req: &Request) -> Response {
        conn.send(req).await.unwrap();
        conn.receive().await.unwrap()
    }

    async fn join_token(conn: &mut Connection<TcpStream>, session_id: Uuid, code: &str) -> String {
        match request(
            conn,
            &Request::Join {
                session_id,
                join_code: code.into(),
            },
        )
        .await
        {
            Response::Joined {
                participant_token, ..
            } => participant_token,
            other => panic!("Expected Joined, got {other:?}"),
        }
    }

    fn event(session_id: Option<Uuid>, token: &str) -> AnalysisEvent {
        AnalysisEvent {
            id: Uuid::new_v4(),
            participant_token: token.into(),
            session_id,
            question_id: "q1".into(),
            concept: "linear-functions".into(),
            misconception: None,
            correct: true,
            hint_count: 0,
            retry_success: false,
            submitted_at: chrono::Utc::now(),
        }
    }

    fn error_message(response: Response) -> String {
        match response {
            Response::Error { message } => message,
            other => panic!("Expected Error, got {other:?}"),
        }
    }

    // ── Wire format / primitives ─────────────────────────────────────────────

    #[test]
    fn request_and_response_use_tagged_snake_case_wire_format() {
        let session_id = Uuid::nil();
        assert_eq!(
            serde_json::to_value(Request::Ping).unwrap(),
            serde_json::json!({"type": "ping"})
        );
        assert_eq!(
            serde_json::to_value(Request::Prepare).unwrap(),
            serde_json::json!({"type": "prepare"})
        );
        assert_eq!(
            serde_json::to_value(Request::Join {
                session_id,
                join_code: "1234".into(),
            })
            .unwrap(),
            serde_json::json!({
                "type": "join",
                "data": {"session_id": session_id, "join_code": "1234"}
            })
        );
        assert_eq!(
            serde_json::to_value(Response::Prepared { session_id }).unwrap(),
            serde_json::json!({"type": "prepared", "data": {"session_id": session_id}})
        );
        assert_eq!(
            serde_json::to_value(Response::Error {
                message: "x".into()
            })
            .unwrap(),
            serde_json::json!({"type": "error", "data": {"message": "x"}})
        );
        let parsed: Request = serde_json::from_value(serde_json::json!({"type": "ping"})).unwrap();
        assert!(matches!(parsed, Request::Ping));
        assert!(serde_json::from_value::<Request>(serde_json::json!({"type": "unknown"})).is_err());
    }

    #[test]
    fn fingerprint_is_deterministic_and_key_specific() {
        let a = keypair().unwrap();
        let b = keypair().unwrap();
        assert_eq!(fingerprint(&a.public), fingerprint(&a.public));
        assert_ne!(fingerprint(&a.public), fingerprint(&b.public));
        assert_eq!(a.public.len(), 32);
        assert_eq!(a.private.len(), 32);
    }

    #[tokio::test]
    async fn write_frame_rejects_empty_and_oversized_records() {
        let (mut a, _b) = tokio::io::duplex(1024);
        assert!(write_frame(&mut a, &[]).await.is_err());
        let oversized = vec![0u8; MAX_RECORD + 1];
        assert!(write_frame(&mut a, &oversized).await.is_err());
    }

    #[tokio::test]
    async fn read_frame_rejects_zero_and_too_long_lengths() {
        let (mut a, mut b) = tokio::io::duplex(1024);
        a.write_all(&[0, 0]).await.unwrap();
        let mut buf = [0u8; 16];
        assert!(read_frame(&mut b, &mut buf).await.is_err());

        let (mut a, mut b) = tokio::io::duplex(1024);
        a.write_all(&[0, 17]).await.unwrap();
        a.write_all(&[0u8; 17]).await.unwrap();
        assert!(read_frame(&mut b, &mut buf).await.is_err());
    }

    #[tokio::test]
    async fn read_frame_fails_on_truncated_stream() {
        let (mut a, mut b) = tokio::io::duplex(1024);
        a.write_all(&[0, 10, 1, 2, 3]).await.unwrap();
        drop(a);
        let mut buf = [0u8; 16];
        assert!(read_frame(&mut b, &mut buf).await.is_err());
    }

    // ── Handshake ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn server_rejects_client_with_wrong_protocol_payload() {
        let server_keys = keypair().unwrap();
        let client_keys = keypair().unwrap();
        let (mut client_io, server_io) = tokio::io::duplex(4 * MAX_RECORD);
        let server = tokio::spawn(async move {
            Connection::handshake_server(server_io, &server_keys.private).await
        });

        let mut noise = Builder::new(params().unwrap())
            .local_private_key(&client_keys.private)
            .unwrap()
            .build_initiator()
            .unwrap();
        let mut out = vec![0u8; MAX_RECORD];
        let n = noise.write_message(b"LUMIN\x01", &mut out).unwrap();
        write_frame(&mut client_io, &out[..n]).await.unwrap();

        let error = server.await.unwrap().err().expect("handshake must fail");
        assert!(error.contains("非対応プロトコル"), "{error}");
    }

    #[tokio::test]
    async fn client_rejects_server_with_wrong_protocol_payload() {
        let server_keys = keypair().unwrap();
        let client_keys = keypair().unwrap();
        let (client_io, mut server_io) = tokio::io::duplex(4 * MAX_RECORD);
        let client = tokio::spawn(async move {
            Connection::handshake_client(client_io, &client_keys.private).await
        });

        let mut noise = Builder::new(params().unwrap())
            .local_private_key(&server_keys.private)
            .unwrap()
            .build_responder()
            .unwrap();
        let mut input = vec![0u8; MAX_RECORD];
        let mut out = vec![0u8; MAX_RECORD];
        let n = read_frame(&mut server_io, &mut input).await.unwrap();
        noise.read_message(&input[..n], &mut out).unwrap();
        let n = noise.write_message(b"OTHER", &mut out).unwrap();
        write_frame(&mut server_io, &out[..n]).await.unwrap();

        let error = client.await.unwrap().err().expect("handshake must fail");
        assert!(error.contains("アプリを更新してください"), "{error}");
    }

    #[tokio::test]
    async fn handshake_fails_when_peer_disconnects() {
        let server_keys = keypair().unwrap();
        let (client_io, server_io) = tokio::io::duplex(1024);
        drop(client_io);
        assert!(
            Connection::handshake_server(server_io, &server_keys.private)
                .await
                .is_err()
        );
    }

    // ── Fragmentation ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn send_rejects_payload_over_json_limit() {
        let (mut client, _server) = connected_pair().await;
        let huge = "A".repeat(MAX_JSON + 1);
        let error = client.send(&huge).await.unwrap_err();
        assert!(error.contains("上限"), "{error}");
    }

    #[tokio::test]
    async fn receive_reassembles_out_of_order_fragments() {
        let (mut client, mut server) = connected_pair().await;
        let id = [7u8; 16];
        send_raw_record(&mut client, &fragment(id, 2, 1, b"\"there\"]")).await;
        send_raw_record(&mut client, &fragment(id, 2, 0, b"[\"hello\",")).await;
        let value: Vec<String> = server.receive().await.unwrap();
        assert_eq!(value, ["hello", "there"]);
    }

    #[tokio::test]
    async fn receive_rejects_short_record() {
        let (mut client, mut server) = connected_pair().await;
        send_raw_record(&mut client, &[0u8; 19]).await;
        let error = server.receive::<serde_json::Value>().await.unwrap_err();
        assert!(error.contains("形式が不正"), "{error}");
    }

    #[tokio::test]
    async fn receive_rejects_invalid_fragment_headers() {
        for (count, index) in [(0u16, 0u16), (2, 2), (19, 0)] {
            let (mut client, mut server) = connected_pair().await;
            send_raw_record(&mut client, &fragment([1u8; 16], count, index, b"{}")).await;
            let error = server.receive::<serde_json::Value>().await.unwrap_err();
            assert!(
                error.contains("順序が不正"),
                "count={count} index={index}: {error}"
            );
        }
    }

    #[tokio::test]
    async fn receive_rejects_fragments_from_different_messages() {
        let (mut client, mut server) = connected_pair().await;
        send_raw_record(&mut client, &fragment([1u8; 16], 2, 0, b"[1,")).await;
        send_raw_record(&mut client, &fragment([2u8; 16], 2, 1, b"2]")).await;
        let error = server.receive::<serde_json::Value>().await.unwrap_err();
        assert!(error.contains("順序が不正"), "{error}");
    }

    #[tokio::test]
    async fn receive_rejects_duplicate_fragment_and_count_change() {
        let (mut client, mut server) = connected_pair().await;
        send_raw_record(&mut client, &fragment([1u8; 16], 2, 0, b"[1,")).await;
        send_raw_record(&mut client, &fragment([1u8; 16], 2, 0, b"[1,")).await;
        let error = server.receive::<serde_json::Value>().await.unwrap_err();
        assert!(error.contains("重複"), "{error}");

        let (mut client, mut server) = connected_pair().await;
        send_raw_record(&mut client, &fragment([1u8; 16], 3, 0, b"[1,")).await;
        send_raw_record(&mut client, &fragment([1u8; 16], 2, 1, b"2]")).await;
        let error = server.receive::<serde_json::Value>().await.unwrap_err();
        assert!(error.contains("重複"), "{error}");
    }

    #[tokio::test]
    async fn receive_rejects_reassembled_payload_over_json_limit() {
        let (mut client, mut server) = connected_pair().await;
        let body = vec![b' '; FRAGMENT_SIZE];
        let writer = async move {
            for index in 0..18u16 {
                let mut out = vec![0u8; MAX_RECORD];
                let chunk = fragment([9u8; 16], 18, index, &body);
                let n = client.cipher.write_message(&chunk, &mut out).unwrap();
                if write_frame(&mut client.stream, &out[..n]).await.is_err() {
                    break;
                }
            }
            client
        };
        let (_client, result) = tokio::join!(writer, server.receive::<serde_json::Value>());
        let error = result.unwrap_err();
        assert!(error.contains("上限"), "{error}");
    }

    #[tokio::test]
    async fn receive_rejects_tampered_ciphertext() {
        let (mut client, mut server) = connected_pair().await;
        let mut out = vec![0u8; MAX_RECORD];
        let n = client
            .cipher
            .write_message(&fragment([1u8; 16], 1, 0, b"{}"), &mut out)
            .unwrap();
        out[n / 2] ^= 0xff;
        write_frame(&mut client.stream, &out[..n]).await.unwrap();
        assert!(server.receive::<serde_json::Value>().await.is_err());
    }

    // ── serve(): classroom request handling ─────────────────────────────────

    #[tokio::test]
    async fn serve_answers_ping_and_exposes_teacher_fingerprint() {
        let session_id = Uuid::new_v4();
        let server = start_server(test_state(Some(session_id), "1234", Some(test_quiz()))).await;
        let key = keypair().unwrap();
        let mut conn = connect(server.port, &key.private).await;
        assert_eq!(fingerprint(&conn.remote_static), server.fingerprint);
        assert!(matches!(
            request(&mut conn, &Request::Ping).await,
            Response::Pong
        ));
        assert!(matches!(
            request(&mut conn, &Request::Ping).await,
            Response::Pong
        ));
        let _ = server.shutdown.send(true);
        server.handle.await.unwrap();
    }

    #[tokio::test]
    async fn serve_closes_connection_when_preparing_without_active_session() {
        let server = start_server(test_state(None, "1234", Some(test_quiz()))).await;
        let key = keypair().unwrap();
        let mut conn = connect(server.port, &key.private).await;
        conn.send(&Request::Prepare).await.unwrap();
        assert!(conn.receive::<Response>().await.is_err());
        let _ = server.shutdown.send(true);
    }

    #[tokio::test]
    async fn serve_returns_error_and_closes_when_quiz_is_missing() {
        let session_id = Uuid::new_v4();
        let state = test_state(Some(session_id), "1234", None);
        let server = start_server(state.clone()).await;
        let key = keypair().unwrap();
        let mut conn = connect(server.port, &key.private).await;
        let response = request(
            &mut conn,
            &Request::Join {
                session_id,
                join_code: "1234".into(),
            },
        )
        .await;
        assert!(error_message(response).contains("クイズが配信されていません"));
        assert!(state.students.read().await.is_empty());
        // Error 応答後はサーバーが接続を閉じる
        conn.send(&Request::Ping).await.ok();
        assert!(conn.receive::<Response>().await.is_err());
        let _ = server.shutdown.send(true);
    }

    #[tokio::test]
    async fn serve_rejects_join_for_wrong_session_even_with_correct_code() {
        let session_id = Uuid::new_v4();
        let state = test_state(Some(session_id), "1234", Some(test_quiz()));
        let server = start_server(state.clone()).await;
        let key = keypair().unwrap();
        let mut conn = connect(server.port, &key.private).await;
        let response = request(
            &mut conn,
            &Request::Join {
                session_id: Uuid::new_v4(),
                join_code: "1234".into(),
            },
        )
        .await;
        assert!(error_message(response).contains("参加コードまたは授業情報が正しくありません"));
        assert!(state.students.read().await.is_empty());
        let _ = server.shutdown.send(true);
    }

    #[tokio::test]
    async fn serve_reuses_token_for_same_student_key_and_issues_new_for_others() {
        let session_id = Uuid::new_v4();
        let state = test_state(Some(session_id), "1234", Some(test_quiz()));
        let server = start_server(state.clone()).await;
        let alice = keypair().unwrap();
        let bob = keypair().unwrap();

        let mut conn = connect(server.port, &alice.private).await;
        let first = join_token(&mut conn, session_id, "1234").await;
        drop(conn);
        let mut conn = connect(server.port, &alice.private).await;
        let again = join_token(&mut conn, session_id, "1234").await;
        assert_eq!(first, again);

        let mut conn = connect(server.port, &bob.private).await;
        let other = join_token(&mut conn, session_id, "1234").await;
        assert_ne!(first, other);

        let roster = state.students.read().await;
        assert_eq!(roster.len(), 2);
        assert!(roster.iter().any(|s| s.participant_token == first));
        assert!(roster.iter().any(|s| s.participant_token == other));
        drop(roster);
        let _ = server.shutdown.send(true);
    }

    #[tokio::test]
    async fn serve_blocks_join_after_repeated_failures_from_same_ip() {
        let session_id = Uuid::new_v4();
        let state = test_state(Some(session_id), "1234", Some(test_quiz()));
        let server = start_server(state.clone()).await;
        let key = keypair().unwrap();

        for _ in 0..20 {
            // 同じ IP の別の鍵にも制限が適用されることを検証する。
            let key = keypair().unwrap();
            let mut conn = connect(server.port, &key.private).await;
            let response = request(
                &mut conn,
                &Request::Join {
                    session_id,
                    join_code: "0000".into(),
                },
            )
            .await;
            assert!(error_message(response).contains("正しくありません"));
        }

        // 正しいコードでもブロック中は拒否される
        let mut conn = connect(server.port, &key.private).await;
        let response = request(
            &mut conn,
            &Request::Join {
                session_id,
                join_code: "1234".into(),
            },
        )
        .await;
        assert!(error_message(response).contains("参加試行が多すぎます"));
        assert!(state.students.read().await.is_empty());
        let _ = server.shutdown.send(true);
    }

    #[tokio::test]
    async fn serve_successful_join_resets_failure_counter() {
        let session_id = Uuid::new_v4();
        let server = start_server(test_state(Some(session_id), "1234", Some(test_quiz()))).await;
        let key = keypair().unwrap();
        let wrong = Request::Join {
            session_id,
            join_code: "0000".into(),
        };

        for _ in 0..19 {
            let mut conn = connect(server.port, &key.private).await;
            assert!(matches!(
                request(&mut conn, &wrong).await,
                Response::Error { .. }
            ));
        }
        let mut conn = connect(server.port, &key.private).await;
        join_token(&mut conn, session_id, "1234").await;

        // 成功でカウンタがリセットされるため、さらに 19 回失敗してもブロックされない
        for _ in 0..19 {
            let mut conn = connect(server.port, &key.private).await;
            let message = error_message(request(&mut conn, &wrong).await);
            assert!(!message.contains("多すぎます"), "{message}");
        }
        let mut conn = connect(server.port, &key.private).await;
        join_token(&mut conn, session_id, "1234").await;
        let _ = server.shutdown.send(true);
    }

    #[tokio::test]
    async fn serve_rejects_analysis_before_join() {
        let session_id = Uuid::new_v4();
        let state = test_state(Some(session_id), "1234", Some(test_quiz()));
        let server = start_server(state.clone()).await;
        let key = keypair().unwrap();
        let mut conn = connect(server.port, &key.private).await;
        let response = request(
            &mut conn,
            &Request::Analysis(event(Some(session_id), "forged-token")),
        )
        .await;
        assert!(error_message(response).contains("参加者または授業を確認できません"));
        assert!(state.analysis_events.lock().await.is_empty());
        let _ = server.shutdown.send(true);
    }

    #[tokio::test]
    async fn serve_rejects_analysis_impersonating_another_student() {
        let session_id = Uuid::new_v4();
        let state = test_state(Some(session_id), "1234", Some(test_quiz()));
        let server = start_server(state.clone()).await;
        let alice = keypair().unwrap();
        let mallory = keypair().unwrap();

        let mut alice_conn = connect(server.port, &alice.private).await;
        let alice_token = join_token(&mut alice_conn, session_id, "1234").await;
        let mut mallory_conn = connect(server.port, &mallory.private).await;
        join_token(&mut mallory_conn, session_id, "1234").await;

        let response = request(
            &mut mallory_conn,
            &Request::Analysis(event(Some(session_id), &alice_token)),
        )
        .await;
        assert!(matches!(response, Response::Error { .. }));
        assert!(state.analysis_events.lock().await.is_empty());
        let _ = server.shutdown.send(true);
    }

    #[tokio::test]
    async fn serve_rejects_analysis_for_wrong_or_ended_session_and_kicked_student() {
        let session_id = Uuid::new_v4();
        let state = test_state(Some(session_id), "1234", Some(test_quiz()));
        let server = start_server(state.clone()).await;
        let key = keypair().unwrap();

        // 別セッション宛て
        let mut conn = connect(server.port, &key.private).await;
        let token = join_token(&mut conn, session_id, "1234").await;
        let response = request(
            &mut conn,
            &Request::Analysis(event(Some(Uuid::new_v4()), &token)),
        )
        .await;
        assert!(matches!(response, Response::Error { .. }));

        // キックされた生徒
        let mut conn = connect(server.port, &key.private).await;
        let token = join_token(&mut conn, session_id, "1234").await;
        state
            .students
            .write()
            .await
            .retain(|s| s.participant_token != token);
        let response = request(
            &mut conn,
            &Request::Analysis(event(Some(session_id), &token)),
        )
        .await;
        assert!(matches!(response, Response::Error { .. }));

        // 授業終了後
        let mut conn = connect(server.port, &key.private).await;
        let token = join_token(&mut conn, session_id, "1234").await;
        *state.active_session.write().await = None;
        let response = request(
            &mut conn,
            &Request::Analysis(event(Some(session_id), &token)),
        )
        .await;
        assert!(matches!(response, Response::Error { .. }));

        assert!(state.analysis_events.lock().await.is_empty());
        let _ = server.shutdown.send(true);
    }

    #[tokio::test]
    async fn serve_acks_and_deduplicates_analysis_events() {
        let session_id = Uuid::new_v4();
        let state = test_state(Some(session_id), "1234", Some(test_quiz()));
        let server = start_server(state.clone()).await;
        let key = keypair().unwrap();
        let mut conn = connect(server.port, &key.private).await;
        let token = join_token(&mut conn, session_id, "1234").await;

        let submitted = event(Some(session_id), &token);
        for _ in 0..2 {
            match request(&mut conn, &Request::Analysis(submitted.clone())).await {
                Response::Ack(ack) => assert_eq!(ack.event_id, submitted.id),
                other => panic!("Expected Ack, got {other:?}"),
            }
        }
        let second = event(Some(session_id), &token);
        assert!(matches!(
            request(&mut conn, &Request::Analysis(second.clone())).await,
            Response::Ack(_)
        ));

        let events = state.analysis_events.lock().await;
        assert_eq!(
            events.iter().map(|e| e.id).collect::<Vec<_>>(),
            [submitted.id, second.id]
        );
        assert!(events.iter().all(|e| e.session_id == Some(session_id)));
        assert!(events.iter().all(|e| e.participant_token == token));
        drop(events);
        let _ = server.shutdown.send(true);
    }

    #[tokio::test]
    async fn serve_shutdown_closes_open_connections_and_stops_listening() {
        let session_id = Uuid::new_v4();
        let server = start_server(test_state(Some(session_id), "1234", Some(test_quiz()))).await;
        let key = keypair().unwrap();
        let mut conn = connect(server.port, &key.private).await;
        assert!(matches!(
            request(&mut conn, &Request::Ping).await,
            Response::Pong
        ));

        server.shutdown.send(true).unwrap();
        server.handle.await.unwrap();

        assert!(conn.receive::<Response>().await.is_err());
        let reconnect = tokio::time::timeout(
            Duration::from_secs(2),
            TcpStream::connect(("127.0.0.1", server.port)),
        )
        .await
        .expect("checking the closed listener should not time out");
        assert!(reconnect.is_err(), "listener should be closed");
    }
}
