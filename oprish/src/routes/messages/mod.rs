use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};
use crate::Cache;
use rocket::serde::json::Json;
use rocket::{Route, State};
use rocket_db_pools::deadpool_redis::redis::AsyncCommands;
use rocket_db_pools::Connection;
use todel::http::{ClientIP, TokenAuth, DB};
use todel::models::{ErrorResponse, Message, MessageCreate, ServerPayload, User};
use todel::Conf;

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
#[autodoc("/messages", category = "Messaging")]
#[post("/", data = "<message>")]
pub async fn create_message(
    message: Json<MessageCreate>,
    mut db: Connection<DB>,
    mut cache: Connection<Cache>,
    conf: &State<Conf>,
    session: TokenAuth,
    ip: ClientIP,
) -> RateLimitedRouteResponse<Result<Json<Message>, ErrorResponse>> {
    let mut rate_limiter = RateLimiter::new("create_message", ip, conf.inner());
    rate_limiter.process_rate_limit(&mut cache).await?;

    let mut message = message.into_inner();
    message.content = message.content.trim().to_string();

    // TODO: handle validation in logic impl
    if message.content.is_empty() || message.content.len() > conf.oprish.message_limit {
        error!(
            rate_limiter,
            VALIDATION,
            "content",
            format!(
                "Message content has to be between 1 and {} characters long",
                conf.oprish.message_limit
            )
        );
    }
    if let Some(disguise) = &message.disguise {
        if let Some(name) = &disguise.name {
            if name.len() < 2 || name.len() > 32 {
                error!(
                    rate_limiter,
                    VALIDATION,
                    "disguise.name",
                    "The user's disguise name must be between 2 and 32 characters in length"
                );
            }
        }
    }

    let payload = ServerPayload::MessageCreate(Message {
        author: User::get(session.0.user_id, None, &mut db, &mut *cache)
            .await
            .map_err(|err| rate_limiter.add_headers(err))?,
        message,
    });
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

pub fn get_routes() -> Vec<Route> {
    routes![create_message]
}
