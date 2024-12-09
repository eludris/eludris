use rocket::{serde::json::Json, State};
use rocket_db_pools::{deadpool_redis::redis::AsyncCommands, Connection};
use todel::{
    http::{Cache, TokenAuth, DB},
    ids::IdGenerator,
    models::{Category, CategoryCreate, ErrorResponse, ServerPayload, Sphere},
    Conf,
};
use tokio::sync::Mutex;

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Create a category for channels inside a sphere.
///
/// -- STATUS: 200
/// -----
///
/// ### Example
///
/// ```sh
/// curl \
///   -H "Authorization: <token>" \
///   --json '{"name":"Bean"}' \
///   https://api.eludris.gay/spheres/1234/categories
/// ```
#[autodoc("/spheres", category = "Spheres")]
#[post("/<sphere_id>/categories", data = "<category>")]
pub async fn create_category(
    category: Json<CategoryCreate>,
    sphere_id: u64,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    id_generator: &State<Mutex<IdGenerator>>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Json<Category>> {
    let mut rate_limiter = RateLimiter::new("create_category", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;

    let sphere = Sphere::get_unpopulated(sphere_id, &mut db)
        .await
        .map_err(|err| {
            rate_limiter.add_headers(if let ErrorResponse::NotFound { .. } = err {
                error!(VALIDATION, "sphere", "Sphere doesn't exist")
            } else {
                err
            })
        })?;

    if sphere.owner_id != session.0.user_id {
        return Err(rate_limiter.add_headers(error!(FORBIDDEN)));
    }

    let category = Category::create(
        category.into_inner(),
        sphere_id,
        &mut *id_generator.lock().await,
        &mut db,
    )
    .await
    .map_err(|err| rate_limiter.add_headers(err))?;

    cache
        .publish::<&str, String, ()>(
            "eludris-events",
            serde_json::to_string(&ServerPayload::CategoryCreate {
                category: category.clone(),
                sphere_id,
            })
            .unwrap(),
        )
        .await
        .unwrap();

    rate_limiter.wrap_response(Json(category))
}
