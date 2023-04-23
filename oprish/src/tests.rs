#[cfg(test)]
mod tests {
    use crate::{rocket, Cache};
    use deadpool_redis::Connection;
    use rocket::{futures::StreamExt, http::Status, local::asynchronous::Client};
    use todel::{
        models::{InstanceInfo, Message, ServerPayload},
        Conf,
    };

    #[rocket::async_test]
    async fn index() {
        let client = Client::untracked(rocket().unwrap()).await.unwrap();
        let conf = &client.rocket().state::<Conf>().unwrap();

        let response = client.get("/").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string().await.unwrap(),
            serde_json::to_string(&InstanceInfo::from_conf(conf, false)).unwrap()
        );

        let response = client.get("/?rate_limits").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string().await.unwrap(),
            serde_json::to_string(&InstanceInfo::from_conf(conf, true)).unwrap()
        );
    }

    #[rocket::async_test]
    async fn send_message() {
        let client = Client::untracked(rocket().unwrap()).await.unwrap();
        let message = Message {
            author: "Woo".to_string(),
            content: "HeWoo there".to_string(),
        };

        let message_str = serde_json::to_string(&message).unwrap();
        let payload = serde_json::to_string(&ServerPayload::MessageCreate(message)).unwrap();

        let pool = client.rocket().state::<Cache>().unwrap();

        let cache = pool.get().await.unwrap();
        let cache = Connection::take(cache);
        let mut cache = cache.into_pubsub();
        cache.subscribe("oprish-events").await.unwrap();

        let response = client
            .post("/messages/")
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
