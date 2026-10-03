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
    mut shutdown: watch::Receiver<bool>,
) -> Result<(tokio::task::JoinHandle<()>, String), String> {
    let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port)))
        .await
        .map_err(err)?;
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
}
