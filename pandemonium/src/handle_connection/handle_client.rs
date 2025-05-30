use futures::stream::{SplitSink, SplitStream};
use futures::StreamExt;
use redis::aio::Connection;
use redis::AsyncCommands;
use sqlx::{Pool, Postgres};
use std::net::IpAddr;
use std::sync::Arc;
use todel::models::{ClientPayload, Secret, ServerPayload, Session, StatusType, User};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Message as WebSocketMessage;
use tokio_tungstenite::WebSocketStream;

use crate::handle_connection::{send_payload, SessionData};
use crate::rate_limit::RateLimiter;

pub async fn handle_client(
    session: Arc<Mutex<Option<SessionData>>>,
    cache: Arc<Mutex<Connection>>,
    rx: &mut SplitStream<WebSocketStream<TcpStream>>,
    tx: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, WebSocketMessage>>>,
    mut rate_limited: bool,
    mut rate_limiter: RateLimiter,
    rl_address: IpAddr,
    pool: Arc<Pool<Postgres>>,
    last_ping: Arc<Mutex<Instant>>,
    secret: Arc<Secret>,
) -> String {
    while let Some(msg) = rx.next().await {
        log::trace!("New gateway message:\n{:#?}", msg);
        if let Err(wait) = rate_limiter.process_rate_limit().await {
            if rate_limited {
                log::debug!(
                    "Disconnected a client: {}, reason: Hit rate_limit",
                    rl_address
                );
                return "Client got ratelimited".to_string();
            } else {
                send_payload(&tx, &ServerPayload::RateLimit { wait }).await;
                rate_limited = true;
            }
        } else if rate_limited {
            rate_limited = false;
        }
        match msg {
            Ok(data) => match data {
                WebSocketMessage::Text(message) => {
                    match serde_json::from_str::<ClientPayload>(&message) {
                        Ok(payload) => {
                            if let Err(err) = handle_payload(
                                payload, &last_ping, &session, &cache, &tx, &pool, &secret,
                            )
                            .await
                            {
                                return err;
                            }
                        }
                        _ => log::debug!("Unknown gateway payload: {}", message),
                    }
                }
                _ => log::debug!("Unsupported Gateway message type."),
            },
            Err(_) => return "Server failed to receive payload".to_string(),
        }
    }
    "Connection unexpectedly died".to_string()
}

async fn handle_payload(
    payload: ClientPayload,
    last_ping: &Arc<Mutex<Instant>>,
    session: &Arc<Mutex<Option<SessionData>>>,
    cache: &Arc<Mutex<Connection>>,
    tx: &Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, WebSocketMessage>>>,
    pool: &Arc<Pool<Postgres>>,
    secret: &Arc<Secret>,
) -> Result<(), String> {
    match payload {
        ClientPayload::Ping => {
            let mut last_ping = last_ping.lock().await;
            *last_ping = Instant::now();
            send_payload(tx, &ServerPayload::Pong).await;
        }
        ClientPayload::Authenticate(token) => {
            let mut session = session.lock().await;
            if session.is_some() {
                return Ok(());
            }
            let mut db = match pool.acquire().await {
                Ok(conn) => conn,
                Err(err) => {
                    log::error!(
                        "Couldn't acquire database connection for Authenticate: {}",
                        err
                    );
                    return Err("Server failed to authenticate client".to_string());
                }
            };
            let user_session = match Session::validate_token(&token, secret, &mut db).await {
                Ok(session) => session,
                Err(_) => return Err("Invalid credentials".to_string()),
            };
            let mut cache = cache.lock().await;
            let sessions: u32 = match cache
                .incr(format!("session:{}", user_session.user_id), 1)
                .await
            {
                Ok(sessions) => sessions,
                Err(err) => {
                    log::error!("Failed to increment user active session counter: {}", err);
                    return Err("Failed to connect user".to_string());
                }
            };
            if sessions == 1 {
                if let Err(err) = cache
                    .sadd::<_, _, ()>("sessions", user_session.user_id)
                    .await
                {
                    log::error!("Failed to add user to online users: {}", err);
                    return Err("Failed to connect user".to_string());
                }
            }
            let user = match User::get_unfiltered(user_session.user_id, &mut db).await {
                Ok(user) => user,
                Err(err) => {
                    log::error!("Failed to get user info: {}", err);
                    return Err("Failed to connect user".to_string());
                }
            };
            if user.status.status_type != StatusType::Offline {
                if let Err(err) = cache
                    .publish::<_, _, ()>(
                        "eludris-events",
                        serde_json::to_string(&ServerPayload::PresenceUpdate {
                            user_id: user_session.user_id,
                            // I don't like this either
                            status: user.status.clone(),
                        })
                        .expect("Couldn't serialize PRESENCE_UPDATE event"),
                    )
                    .await
                {
                    log::error!("Failed to publish PRESENCE_UPDATE: {}", err);
                    return Err("Failed to connect user".to_string());
                };
            }
            let spheres = user
                .get_spheres(&mut db, &mut *cache)
                .await
                .map_err(|_| "Failed to connect user".to_string())?;
            let payload = ServerPayload::Authenticated { user, spheres };
            send_payload(tx, &payload).await;
            if let ServerPayload::Authenticated { user, spheres } = payload {
                *session = Some(SessionData {
                    session: user_session,
                    user,
                    sphere_ids: spheres.iter().map(|s| s.id).collect(),
                });
            }
        }
    }
    Ok(())
}
