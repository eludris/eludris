use argon2::Argon2;
use rocket::{http::Status, serde::json::Json, Route, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, ClientIP, DB},
    ids::IdGenerator,
    models::{Secret, Session, SessionCreate, SessionCreated},
    Conf,
};
use tokio::sync::Mutex;

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

#[post("/", data = "<session>")]
pub async fn create_session(
    session: Json<SessionCreate>,
    verifier: &State<Argon2<'static>>,
    id_generator: &State<Mutex<IdGenerator>>,
    secret: &State<Secret>,
    conf: &State<Conf>,
    mut db: Connection<DB>,
    mut cache: Connection<Cache>,
    ip: ClientIP,
) -> RateLimitedRouteResponse<(Status, Json<SessionCreated>)> {
    let mut rate_limiter = RateLimiter::new("create_session", &ip, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;

    rate_limiter.wrap_response((
        Status::Created,
        Json(
            Session::create(
                session.into_inner(),
                *ip,
                secret,
                verifier.inner(),
                &mut *id_generator.lock().await,
                &mut db,
            )
            .await
            .map_err(|err| rate_limiter.add_headers(err))?,
        ),
    ))
}

pub fn get_routes() -> Vec<Route> {
    routes![create_session]
}