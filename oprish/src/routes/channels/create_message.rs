use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};
use crate::Cache;
use rocket::serde::json::Json;
use rocket::State;
use rocket_db_pools::deadpool_redis::redis::AsyncCommands;
use rocket_db_pools::Connection;
use todel::http::{TokenAuth, DB};
use todel::ids::IdGenerator;
use todel::models::{ErrorResponse, Message, MessageCreate, ServerPayload, SphereChannel};
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
) -> RateLimitedRouteResponse<Result<Json<Message>, ErrorResponse>> {
    let mut rate_limiter = RateLimiter::new(
        "create_message",
        format!("{}:{}", channel_id, session.0.user_id),
        conf.inner(),
    );
    rate_limiter.process_rate_limit(&mut cache).await?;

    if !SphereChannel::has_member(channel_id, session.0.user_id, &mut db)
        .await
        .map_err(|err| rate_limiter.add_headers(err))?
    {
        error!(rate_limiter, UNAUTHORIZED);
    }

    let mut cache = cache.into_inner();
    let message = Message::create(
        message.into_inner(),
        channel_id,
        session.0.user_id,
        &mut *id_generator.lock().await,
        &mut db,
        &mut cache,
    )
    .await
    .map_err(|err| rate_limiter.add_headers(err))?;

    cache
        .publish::<&str, String, ()>(
            "eludris-events",
            serde_json::to_string(&ServerPayload::MessageCreate(message.clone())).unwrap(),
        )
        .await
        .unwrap();

    let message_clone = message.clone();
    tokio::spawn(async move { message_clone.populate_embeds(db.into_inner(), cache).await });

    rate_limiter.wrap_response(Ok(Json(message)))
}
