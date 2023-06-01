pub mod messages;

use argon2::Argon2;
use rand::rngs::StdRng;
use rocket::{serde::json::Json, Route, State};
use rocket_db_pools::Connection;
use todel::{
    http::{ClientIP, TokenAuth, DB},
    ids::IdGenerator,
    models::{
        Emailer, ErrorResponse, InstanceInfo, Secret, Session, SessionCreate, SessionCreated, User,
        UserCreate,
    },
    Conf,
};
use tokio::sync::Mutex;

use crate::{
    rate_limit::{RateLimitedRouteResponse, RateLimiter},
    Cache,
}; // poggers

/// Get information about the instance you're sending this request to.
///
/// Most of this data comes from the instance's configuration.
///
/// -----
///
/// ### Example
///
/// ```sh
/// curl https://api.eludris.gay/?rate_limits
///
/// {
///   "instance_name": "eludris",
///   "description": "The *almost* official Eludris instance - ooliver.",
///   "version": "0.3.2",
///   "message_limit": 2000,
///   "oprish_url": "https://api.eludris.gay",
///   "pandemonium_url": "wss://ws.eludris.gay/",
///   "effis_url": "https://cdn.eludris.gay",
///   "file_size": 20000000,
///   "attachment_file_size": 25000000,
///   "rate_limits": {
///     "oprish": {
///       "info": {
///         "reset_after": 5,
///         "limit": 2
///       },
///       "message_create": {
///         "reset_after": 5,
///         "limit": 10
///       }
///     },
///     "pandemonium": {
///       "reset_after": 10,
///       "limit": 5
///     },
///     "effis": {
///       "assets": {
///         "reset_after": 60,
///         "limit": 5,
///         "file_size_limit": 30000000
///       },
///       "attachments": {
///         "reset_after": 180,
///         "limit": 20,
///         "file_size_limit": 500000000
///       },
///       "fetch_file": {
///         "reset_after": 60,
///         "limit": 30
///       }
///     }
///   }
/// }
/// ```
#[autodoc(category = "Instance")]
#[get("/?<rate_limits>")]
pub async fn get_instance_info(
    rate_limits: bool,
    address: ClientIP,
    mut cache: Connection<Cache>,
    conf: &State<Conf>,
) -> RateLimitedRouteResponse<Json<InstanceInfo>> {
    let mut rate_limiter = RateLimiter::new("get_instance_info", address, conf.inner());
    rate_limiter.process_rate_limit(&mut cache).await?;

    rate_limiter.wrap_response(Json(InstanceInfo::from_conf(conf.inner(), rate_limits)))
}

#[post("/signup", data = "<user>")]
pub async fn signup(
    user: Json<UserCreate>,
    hasher: &State<Argon2<'static>>,
    rng: &State<Mutex<StdRng>>,
    id_generator: &State<Mutex<IdGenerator>>,
    conf: &State<Conf>,
    mailer: &State<Emailer>,
    mut db: Connection<DB>,
    cache: Connection<Cache>,
) -> Result<Json<User>, ErrorResponse> {
    Ok(Json(
        User::create(
            user.into_inner(),
            hasher.inner(),
            &mut *rng.lock().await,
            &mut *id_generator.lock().await,
            conf,
            mailer,
            &mut db,
            &mut cache.into_inner(),
        )
        .await?,
    ))
}

#[post("/login", data = "<session>")]
async fn login(
    session: Json<SessionCreate>,
    verifier: &State<Argon2<'static>>,
    id_generator: &State<Mutex<IdGenerator>>,
    secret: &State<Secret>,
    mut db: Connection<DB>,
    ip: ClientIP,
) -> Result<Json<SessionCreated>, ErrorResponse> {
    Ok(Json(
        Session::create(
            session.into_inner(),
            *ip,
            secret,
            verifier.inner(),
            &mut *id_generator.lock().await,
            &mut db,
        )
        .await?,
    ))
}

#[post("/verify?<code>")]
async fn verify(
    code: u32,
    mut db: Connection<DB>,
    cache: Connection<Cache>,
    session: TokenAuth,
) -> Result<(), ErrorResponse> {
    User::verify(code, session.0, &mut db, &mut cache.into_inner()).await?;
    Ok(())
}

pub fn get_routes() -> Vec<Route> {
    routes![get_instance_info, signup, login, verify]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rocket;
    use rocket::{http::Status, local::asynchronous::Client};
    use todel::{models::InstanceInfo, Conf};

    #[rocket::async_test]
    async fn index() {
        let client = Client::untracked(rocket().unwrap()).await.unwrap();
        let conf = &client.rocket().state::<Conf>().unwrap();

        let response = client
            .get(uri!(get_instance_info(rate_limits = false)))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string().await.unwrap(),
            serde_json::to_string(&InstanceInfo::from_conf(conf, false)).unwrap()
        );

        let response = client
            .get(uri!(get_instance_info(rate_limits = true)))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string().await.unwrap(),
            serde_json::to_string(&InstanceInfo::from_conf(conf, true)).unwrap()
        );
    }
}
