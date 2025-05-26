use rocket::{serde::json::Json, State};
use rocket_db_pools::{deadpool_redis::redis::AsyncCommands, Connection};
use todel::{
    http::{Cache, SphereIdentifier, TokenAuth, DB},
    models::{ErrorResponse, ServerPayload, Sphere, SphereEdit},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

#[autodoc("/spheres", category = "Spheres")]
#[patch("/<identifier>", data = "<edit>")]
pub async fn edit(
    edit: Json<SphereEdit>,
    identifier: SphereIdentifier,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Json<Sphere>> {
    let mut rate_limiter = RateLimiter::new("edit_channel", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;

    let sphere_id = match identifier {
        SphereIdentifier::ID(id) => id,
        SphereIdentifier::Slug(slug) => {
            Sphere::get_unpopulated_slug(slug, &mut db)
                .await
                .map_err(|err| rate_limiter.add_headers(err))?
                .id
        }
    };

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

    let sphere = Sphere::edit(edit.clone().into_inner(), sphere_id, &mut db)
        .await
        .map_err(|err| rate_limiter.add_headers(err))?;

    cache
        .publish::<&str, String, ()>(
            "eludris-events",
            serde_json::to_string(&ServerPayload::SphereUpdate {
                data: edit.into_inner(),
                sphere_id,
            })
            .unwrap(),
        )
        .await
        .unwrap();

    rate_limiter.wrap_response(Json(sphere))
}
