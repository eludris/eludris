use rocket::{serde::json::Json, State};
use rocket_db_pools::{deadpool_redis::redis::AsyncCommands, Connection};
use todel::{
    http::{Cache, TokenAuth, DB},
    models::{Category, CategoryEdit, ServerPayload},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Edit a category.
///
/// -- STATUS: 201
/// -----
///
/// ### Example
///
/// ```sh
/// curl --request PATCH \
///   -H "Authorization: <token>" \
///   --json '{"name":"Bean","position":42}' \
///   https://api.eludris.gay/spheres/1234/categories/5678
/// ```
#[autodoc("/spheres", category = "Spheres")]
#[patch("/<sphere_id>/categories/<category_id>", data = "<category>")]
pub async fn edit_category(
    category: Json<CategoryEdit>,
    sphere_id: u64,
    category_id: u64,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Json<Category>> {
    let mut rate_limiter = RateLimiter::new("edit_category", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;
    let category = category.into_inner();

    cache
        .publish::<&str, String, ()>(
            "eludris-events",
            serde_json::to_string(&ServerPayload::CategoryEdit {
                data: category.clone(),
                category_id,
                sphere_id,
            })
            .unwrap(),
        )
        .await
        .unwrap();

    rate_limiter.wrap_response(Json(
        Category::edit(category, sphere_id, category_id, &mut db)
            .await
            .map_err(|err| rate_limiter.add_headers(err))?,
    ))
}
