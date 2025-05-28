mod handle_client;
mod handle_pubsub;

use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use redis::aio::Connection;
use redis::aio::PubSub;
use redis::AsyncCommands;
use sqlx::{Pool, Postgres};
use std::borrow::Cow;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use todel::models::{InstanceInfo, Secret, ServerPayload, Session, Status, StatusType, User};
use todel::Conf;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{interval, Instant};
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message as WebSocketMessage;
use tokio_tungstenite::{accept_hdr_async, WebSocketStream};

use crate::handle_connection::handle_client::handle_client;
use crate::handle_connection::handle_pubsub::handle_pubsub;
use crate::rate_limit::RateLimiter;

// /// Some padding to account for network latency.
// const TIMEOUT_PADDING: Duration = Duration::from_secs(3);

/// Actual timeout duration.
const TIMEOUT: Duration = Duration::from_secs(45);

/// The duration it takes for a connection to be inactive in for it to be regarded as zombified and
/// disconnected.
const TIMEOUT_DURATION: Duration = Duration::from_secs(48); // TIMEOUT_PADDING

/// Internal pandemonium specific-struct for stored user session-related data.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionData {
    session: Session,
    user: User,
    sphere_ids: Vec<u64>,
}

/// A simple function that check's if a client's last ping was over TIMEOUT_DURATION seconds ago and
/// closes the gateway connection if so.
async fn check_connection(last_ping: Arc<Mutex<Instant>>) {
    let mut interval = interval(TIMEOUT_DURATION);
    loop {
        if Instant::now().duration_since(*last_ping.lock().await) > TIMEOUT_DURATION {
            break;
        }
        interval.tick().await;
    }
}

// TODO: (like really to fucking do): split this into it's own helper functions (and sanify code)
/// A function that handles one client connecting and disconnecting.
pub async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    cache: Arc<Mutex<Connection>>,
    pubsub: PubSub,
    pool: Arc<Pool<Postgres>>,
    conf: Arc<Conf>,
    secret: Arc<Secret>,
) {
    let mut rl_address = IpAddr::from_str("127.0.0.1").unwrap();

    let res = accept_hdr_async(stream, |req: &Request, resp: Response| {
        let headers = req.headers();

        if let Some(ip) = headers.get("X-Real-Ip") {
            rl_address = ip
                .to_str()
                .map(|ip| IpAddr::from_str(ip).unwrap_or_else(|_| addr.ip()))
                .unwrap_or_else(|_| addr.ip());
        } else if let Some(ip) = headers.get("CF-Connecting-IP") {
            rl_address = ip
                .to_str()
                .map(|ip| IpAddr::from_str(ip).unwrap_or_else(|_| addr.ip()))
                .unwrap_or_else(|_| addr.ip());
        } else {
            rl_address = addr.ip();
        }

        Ok(resp)
    })
    .await;

    let socket = match res {
        Ok(socket) => socket,
        Err(err) => {
            log::error!(
                "Could not establish a websocket connection with {}: {}",
                rl_address,
                err
            );
            return;
        }
    };

    let (tx, mut rx) = socket.split();
    let tx = Arc::new(Mutex::new(tx));
    let last_ping = Arc::new(Mutex::new(Instant::now()));

    let mut rate_limiter = RateLimiter::new(
        Arc::clone(&cache),
        rl_address,
        Duration::from_secs(conf.pandemonium.rate_limit.reset_after as u64),
        conf.pandemonium.rate_limit.limit,
    );
    let mut rate_limited = false;
    if let Err(wait) = rate_limiter.process_rate_limit().await {
        send_payload(&tx, &ServerPayload::RateLimit { wait }).await;
        rate_limited = true;
    }

    send_payload(
        &tx,
        &ServerPayload::Hello {
            heartbeat_interval: TIMEOUT.as_millis() as u64,
            instance_info: Box::new(InstanceInfo::from_conf(&conf, false)),
            rate_limit: conf.pandemonium.rate_limit.clone(),
        },
    )
    .await;

    let session = Arc::new(Mutex::new(None::<SessionData>));

    tokio::select! {
        _ = check_connection(last_ping.clone()) => {
            log::debug!("Dead connection with client {}", rl_address);
            close_socket(tx, rx, CloseFrame { code: CloseCode::Error, reason: Cow::Borrowed("Client connection dead") }, rl_address).await
        }
        reason = handle_client(
            Arc::clone(&session),
            Arc::clone(&cache),
            &mut rx,
            Arc::clone(&tx),
            rate_limited,
            rate_limiter,
            rl_address,
            Arc::clone(&pool),
            Arc::clone(&last_ping),
            Arc::clone(&secret)
          ) => {
            close_socket(tx, rx, CloseFrame { code: CloseCode::Error, reason: Cow::Owned(reason) }, rl_address).await;
        },
        _ = handle_pubsub(pubsub, Arc::clone(&session), Arc::clone(&cache), Arc::clone(&tx), pool) => {
            close_socket(tx, rx, CloseFrame { code: CloseCode::Error, reason: Cow::Borrowed("Server Error") }, rl_address).await;
        },
    };

    let mut cache = cache.lock().await;
    let session = session.lock().await;
    if session.is_some() {
        let session = session.as_ref().unwrap();
        let sessions: u32 = match cache.decr(format!("session:{}", session.user.id), 1).await {
            Ok(sessions) => sessions,
            Err(err) => {
                log::error!("Failed to decrement user active session counter: {}", err);
                return;
            }
        };
        if sessions == 0 {
            if let Err(err) = cache.srem::<_, _, ()>("sessions", session.user.id).await {
                log::error!("Failed to remove user from online users: {}", err);
            }
            if session.user.status.status_type != StatusType::Offline {
                if let Err(err) = cache
                    .publish::<_, _, ()>(
                        "eludris-events",
                        serde_json::to_string(&ServerPayload::PresenceUpdate {
                            user_id: session.user.id,
                            status: Status {
                                status_type: StatusType::Offline,
                                text: None,
                            },
                        })
                        .expect("Couldn't serialize PRESENCE_UPDATE event"),
                    )
                    .await
                {
                    log::error!("Failed to publish PRESENCE_UPDATE: {}", err);
                };
            }
        }
    }
}

async fn close_socket(
    tx: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, WebSocketMessage>>>,
    rx: SplitStream<WebSocketStream<TcpStream>>,
    frame: CloseFrame<'_>,
    rl_address: IpAddr,
) {
    let tx = Arc::try_unwrap(tx).expect("Couldn't obtain tx from MutexLock");
    let tx = tx.into_inner();

    if let Err(err) = tx
        .reunite(rx)
        .expect("Couldn't reunite WebSocket stream")
        .close(Some(frame))
        .await
    {
        log::debug!("Couldn't close socket with {}: {}", rl_address, err);
    }
}

async fn send_payload(
    tx: &Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, WebSocketMessage>>>,
    payload: &ServerPayload,
) {
    if let Err(err) = tx
        .lock()
        .await
        .send(WebSocketMessage::Text(
            serde_json::to_string(payload).unwrap(),
        ))
        .await
    {
        log::error!("Could not send payload: {}", err);
    }
}
