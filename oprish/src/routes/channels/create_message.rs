use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};
use crate::Cache;
use rocket::serde::json::Json;
use rocket::State;
use rocket_db_pools::deadpool_redis::redis::AsyncCommands;
use rocket_db_pools::Connection;
use todel::http::{ClientIP, TokenAuth, DB};
use todel::ids::IdGenerator;
use todel::models::{ErrorResponse, Message, MessageCreate, ServerPayload};
use todel::Conf;
use tokio::sync::Mutex;

/// Post a message to Eludris.
///
/// -- STATUS: 201
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   --json '{"author":"Not a weeb","content":"Hello, World!"}' \
///   https://api.eludris.gay/messages
///
/// {
///   "author": "Not a weeb",
///   "content": "Hello, World!"
/// }
/// ```
#[autodoc("/channels", category = "Messaging")]
#[post("/<channel_id>/messages", data = "<message>")]
pub async fn create_message(
    channel_id: u64,
    message: Json<MessageCreate>,
    mut db: Connection<DB>,
    mut cache: Connection<Cache>,
    conf: &State<Conf>,
    id_generator: &State<Mutex<IdGenerator>>,
    session: TokenAuth,
    ip: ClientIP,
) -> RateLimitedRouteResponse<Result<Json<Message>, ErrorResponse>> {
    let mut rate_limiter = RateLimiter::new("create_message", ip, conf.inner());
    rate_limiter.process_rate_limit(&mut cache).await?;
    let mut cache = cache.into_inner();
    let payload = ServerPayload::MessageCreate(
        Message::create(
            message.into_inner(),
            channel_id,
            session.0.user_id,
            &mut *id_generator.lock().await,
            &mut db,
            conf,
            &mut cache,
        )
        .await
        .map_err(|err| rate_limiter.add_headers(err))?,
    );

    cache
        .publish::<&str, String, ()>("eludris-events", serde_json::to_string(&payload).unwrap())
        .await
        .unwrap();
    if let ServerPayload::MessageCreate(message) = payload {
        rate_limiter.wrap_response(Ok(Json(message)))
    } else {
        unreachable!()
    }
}
