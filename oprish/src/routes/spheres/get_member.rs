use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, ClientIP, SphereIdentifier, TokenAuth, UserIdentifier, DB},
    models::{ErrorResponse, Member, Sphere},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Get a member's data using a [`SphereIdentifier`] and [`UserIdentifier`].
///
/// -----
///
/// ### Example
///
/// ```sh
/// # TODO: add example
/// ```
#[autodoc("/spheres", category = "Members")]
#[get("/<sphere_identifier>/members/<member_identifier>")]
pub async fn get_member(
    sphere_identifier: SphereIdentifier,
    member_identifier: UserIdentifier,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: Option<TokenAuth>,
    ip: ClientIP,
) -> RateLimitedRouteResponse<Json<Member>> {
    let mut rate_limiter;
    if let Some(session) = &session {
        rate_limiter = RateLimiter::new("get_member", session.0.user_id, conf);
    } else {
        rate_limiter = RateLimiter::new("guest_get_member", ip, conf);
    }
    rate_limiter.process_rate_limit(&mut cache).await?;
    let sphere_id = match sphere_identifier {
        SphereIdentifier::ID(id) => id,
        SphereIdentifier::Slug(slug) => {
            Sphere::get_unpopulated_slug(slug, &mut db)
                .await
                .map_err(|err| rate_limiter.add_headers(err))?
                .id
        }
    };
    let member = match member_identifier {
        UserIdentifier::Me => match session {
            Some(session) => {
                Member::get(
                    session.0.user_id,
                    sphere_id,
                    Some(session.0.user_id),
                    &mut db,
                    &mut cache.into_inner(),
                )
                .await
            }
            None => Err(error!(UNAUTHORIZED)),
        },
        UserIdentifier::ID(id) => {
            Member::get(
                id,
                sphere_id,
                session.map(|s| s.0.user_id),
                &mut db,
                &mut cache.into_inner(),
            )
            .await
        }
        UserIdentifier::Username(username) => {
            Member::get_username(
                &username,
                sphere_id,
                session.map(|s| s.0.user_id),
                &mut db,
                &mut cache.into_inner(),
            )
            .await
        }
    }
    .map_err(|err| rate_limiter.add_headers(err))?;
    rate_limiter.wrap_response(Json(member))
}
