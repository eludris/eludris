use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};
use crate::Cache;
use deadpool_redis::redis::AsyncCommands;
use rocket::serde::json::Json;
use rocket::{Route, State};
use rocket_db_pools::Connection;
use todel::http::ClientIP;
use todel::models::{ErrorResponse, Message, ServerPayload};
use todel::Conf;

#[autodoc("/messages", category = "Messaging")]
#[post("/", data = "<message>")]
pub async fn create_message(
    message: Json<Message>,
    address: ClientIP,
    mut cache: Connection<Cache>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<Result<Json<Message>, ErrorResponse>> {
    let mut rate_limiter = RateLimiter::new("message_create", address, conf.inner());
    rate_limiter.process_rate_limit(&mut cache).await?;

    let message = message.into_inner();
    if message.author.len() < 2 || message.author.len() > 32 {
        error!(
            rate_limiter,
            VALIDATION, "author", "Message author has to be between 2 and 32 characters long"
        );
    } else if message.content.is_empty() || message.content.len() > conf.oprish.message_limit {
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

    let payload = ServerPayload::MessageCreate(message);
    cache
        .publish::<&str, String, ()>("oprish-events", serde_json::to_string(&payload).unwrap())
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
