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

    let payload = ServerPayload::MessageCreate(Message {
        author: User::get(session.0.user_id, None, &mut db, &mut *cache)
            .await
            .map_err(|err| rate_limiter.add_headers(err))?,
        content: message.content,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rocket, Cache};
    use rocket::{futures::StreamExt, http::Status, local::asynchronous::Client};
    use rocket_db_pools::deadpool_redis::Connection;
    use todel::models::{Message, ServerPayload};

    #[rocket::async_test]
    async fn create_message() {
        let client = Client::untracked(rocket().unwrap()).await.unwrap();
        let message = Message {
            content: "HeWoo there".to_string(),
        };

        let message_str = serde_json::to_string(&message).unwrap();
        let payload = serde_json::to_string(&ServerPayload::MessageCreate(message)).unwrap();

        let pool = client.rocket().state::<Cache>().unwrap();

        let cache = pool.get().await.unwrap();
        let mut cache = Connection::take(cache).into_pubsub();
        cache.subscribe("eludris-events").await.unwrap();

        let response = client
            .post(uri!("/messages", create_message))
            .body(&message_str)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().await.unwrap(), message_str);

        assert_eq!(
            cache
                .into_on_message()
                .next()
                .await
                .unwrap()
                .get_payload::<String>()
                .unwrap(),
            payload
        );
    }
}
