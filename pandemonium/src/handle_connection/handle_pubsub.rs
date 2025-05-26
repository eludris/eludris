use futures::stream::SplitSink;
use futures::StreamExt;
use redis::aio::Connection;
use redis::aio::PubSub;
use redis::AsyncCommands;
use sqlx::Pool;
use sqlx::Postgres;
use std::sync::Arc;
use todel::models::{ServerPayload, Sphere, SphereChannel, StatusType};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message as WebSocketMessage;
use tokio_tungstenite::WebSocketStream;

use crate::utils::deserialize_message;

use super::{send_payload, SessionData};

pub async fn handle_pubsub(
    pubsub: PubSub,
    session: Arc<Mutex<Option<SessionData>>>,
    cache: Arc<Mutex<Connection>>,
    tx: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, WebSocketMessage>>>,
    pool: Arc<Pool<Postgres>>,
) {
    pubsub
        .into_on_message()
        .for_each(|msg| async {
            let mut session = session.lock().await;
            if session.is_none() {
                return;
            }
            let session = session.as_mut().unwrap();
            match deserialize_message(msg) {
                Ok(payload) => handle_event(payload, session, &cache, &tx, &pool).await,
                Err(err) => log::warn!("Failed to deserialize event payload: {}", err),
            }
        })
        .await;
}

async fn handle_event(
    payload: ServerPayload,
    session: &mut SessionData,
    cache: &Arc<Mutex<Connection>>,
    tx: &Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, WebSocketMessage>>>,
    pool: &Arc<Pool<Postgres>>,
) {
    match payload {
        ServerPayload::PresenceUpdate { user_id, status } => {
            if user_id == session.user.id {
                session.user.status = status.clone();
            }
            send_payload(tx, &ServerPayload::PresenceUpdate { user_id, status }).await;
        }
        ServerPayload::UserUpdate(mut user) => {
            if user.id == session.user.id {
                session.user = user.clone();
            }
            if user.id != session.user.id {
                user.email = None;
                user.verified = None;
                let sessions: u32 =
                    match cache.lock().await.get(format!("session:{}", user.id)).await {
                        Ok(sessions) => sessions,
                        Err(err) => {
                            log::error!("Failed to get user active session counter: {}", err);
                            return;
                        }
                    };
                if sessions == 0 {
                    user.status.status_type = StatusType::Offline;
                }
                if user.status.status_type == StatusType::Offline {
                    user.status.text = None;
                }
            }
            send_payload(tx, &ServerPayload::UserUpdate(user)).await;
        }
        ServerPayload::SphereMemberJoin { user, sphere_id } => {
            if user.id == session.user.id {
                let mut db = match pool.acquire().await {
                    Ok(conn) => conn,
                    Err(err) => {
                        log::error!(
                            "Couldn't acquire database connection for SphereMemberJoin: {}",
                            err
                        );
                        return;
                    }
                };
                let sphere = match Sphere::get(sphere_id, &mut db, &mut *cache.lock().await).await {
                    Ok(sphere) => sphere,
                    Err(err) => {
                        log::error!("Couldn't fetch sphere data for SphereMemberJoin: {}", err);
                        return;
                    }
                };
                session.sphere_ids.push(sphere_id);
                send_payload(tx, &ServerPayload::SphereJoin(sphere)).await;
            } else if session.sphere_ids.contains(&sphere_id) {
                send_payload(tx, &ServerPayload::SphereMemberJoin { user, sphere_id }).await;
            }
        }
        ServerPayload::MessageCreate(message) => {
            if let SphereChannel::Text(channel) = &message.channel {
                if session.sphere_ids.contains(&channel.sphere_id) {
                    send_payload(tx, &ServerPayload::MessageCreate(message)).await;
                }
            }
        }
        ServerPayload::CategoryCreate {
            category,
            sphere_id,
        } => {
            if session.sphere_ids.contains(&sphere_id) {
                send_payload(
                    tx,
                    &ServerPayload::CategoryCreate {
                        category,
                        sphere_id,
                    },
                )
                .await;
            }
        }
        ServerPayload::CategoryUpdate {
            data,
            category_id,
            sphere_id,
        } => {
            if session.sphere_ids.contains(&sphere_id) {
                send_payload(
                    tx,
                    &ServerPayload::CategoryUpdate {
                        data,
                        category_id,
                        sphere_id,
                    },
                )
                .await;
            }
        }
        ServerPayload::CategoryDelete {
            category_id,
            sphere_id,
        } => {
            if session.sphere_ids.contains(&sphere_id) {
                send_payload(
                    tx,
                    &ServerPayload::CategoryDelete {
                        category_id,
                        sphere_id,
                    },
                )
                .await;
            }
        }
        ServerPayload::SphereChannelCreate { channel, sphere_id } => {
            if session.sphere_ids.contains(&sphere_id) {
                send_payload(
                    tx,
                    &ServerPayload::SphereChannelCreate { channel, sphere_id },
                )
                .await;
            }
        }
        ServerPayload::SphereChannelUpdate {
            data,
            channel_id,
            sphere_id,
        } => {
            if session.sphere_ids.contains(&sphere_id) {
                send_payload(
                    tx,
                    &ServerPayload::SphereChannelUpdate {
                        data,
                        channel_id,
                        sphere_id,
                    },
                )
                .await;
            }
        }
        ServerPayload::SphereChannelDelete {
            channel_id,
            sphere_id,
        } => {
            if session.sphere_ids.contains(&sphere_id) {
                send_payload(
                    tx,
                    &ServerPayload::SphereChannelDelete {
                        channel_id,
                        sphere_id,
                    },
                )
                .await;
            }
        }
        payload => {
            send_payload(tx, &payload).await;
        }
    }
}
