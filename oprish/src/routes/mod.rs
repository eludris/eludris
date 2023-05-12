pub mod messages;

use rocket::{serde::json::Json, Route, State};
use rocket_db_pools::Connection;
use todel::{http::ClientIP, models::InstanceInfo, Conf};

use crate::{
    rate_limit::{RateLimitedRouteResponse, RateLimiter},
    Cache,
}; // poggers

#[autodoc(category = "Instance")]
#[get("/?<rate_limits>")]
pub async fn get_instance_info(
    rate_limits: bool,
    address: ClientIP,
    mut cache: Connection<Cache>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<Json<InstanceInfo>> {
    let mut rate_limiter = RateLimiter::new("info", address, conf.inner());
    rate_limiter.process_rate_limit(&mut cache).await?;

    rate_limiter.wrap_response(Json(InstanceInfo::from_conf(conf.inner(), rate_limits)))
}

pub fn get_routes() -> Vec<Route> {
    routes![get_instance_info]
}

#[cfg(test)]
mod tests {
    use crate::{rocket};
    use rocket::{http::Status, local::asynchronous::Client};
    use todel::{
        models::{InstanceInfo},
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
}
