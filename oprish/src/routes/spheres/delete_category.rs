use rocket::State;
use rocket_db_pools::{deadpool_redis::redis::AsyncCommands, Connection};
use todel::{
    http::{Cache, TokenAuth, DB},
    models::{Category, ErrorResponse, ServerPayload, Sphere},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Delete a category.
///
/// -- STATUS: 201
/// -----
///
/// ### Example
///
/// ```sh
/// curl --request DELETE \
///   -H "Authorization: <token>" \
///   https://api.eludris.gay/spheres/1234/categories/5678
/// ```
#[autodoc("/spheres", category = "Spheres")]
#[delete("/<sphere_id>/categories/<category_id>")]
pub async fn delete_category(
    sphere_id: u64,
    category_id: u64,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<()> {
    let mut rate_limiter = RateLimiter::new("edit_category", session.0.user_id, conf);
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

    let response = rate_limiter.wrap_response(
        Category::delete(sphere_id, category_id, &mut db)
            .await
            .map_err(|err| rate_limiter.add_headers(err))?,
    );

    cache
        .publish::<&str, String, ()>(
            "eludris-events",
            serde_json::to_string(&ServerPayload::CategoryDelete {
                category_id,
                sphere_id,
            })
            .unwrap(),
        )
        .await
        .unwrap();

    response
}
