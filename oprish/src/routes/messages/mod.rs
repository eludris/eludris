use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};
use crate::Cache;
use deadpool_redis::redis::AsyncCommands;
use rocket::serde::json::Json;
use rocket::{Route, State};
use rocket_db_pools::Connection;
use todel::http::ClientIP;
use todel::models::{ErrorResponse, Message, ServerPayload};
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
    message: Json<Message>,
    address: ClientIP,
    mut cache: Connection<Cache>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<Result<Json<Message>, ErrorResponse>> {
    let mut rate_limiter = RateLimiter::new("message_create", address, conf.inner());
    rate_limiter.process_rate_limit(&mut cache).await?;

    let mut message = message.into_inner();
    message.author = message.author.trim().to_string();
    message.content = message.content.trim().to_string();

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rocket, Cache};
    use deadpool_redis::Connection;
    use rocket::{futures::StreamExt, http::Status, local::asynchronous::Client};
    use todel::models::{Message, ServerPayload};

    #[rocket::async_test]
    async fn create_message() {
        let client = Client::untracked(rocket().unwrap()).await.unwrap();
        let message = Message {
            author: "Woo".to_string(),
            content: "HeWoo there".to_string(),
        };

        let message_str = serde_json::to_string(&message).unwrap();
        let payload = serde_json::to_string(&ServerPayload::MessageCreate(message)).unwrap();

        let pool = client.rocket().state::<Cache>().unwrap();

        let cache = pool.get().await.unwrap();
        let mut cache = Connection::take(cache).into_pubsub();
        cache.subscribe("oprish-events").await.unwrap();

        let response = client
            .post(uri!(create_message))
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
