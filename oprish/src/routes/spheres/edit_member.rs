use rocket::{serde::json::Json, State};
use rocket_db_pools::Connection;
use todel::{
    http::{Cache, SphereIdentifier, TokenAuth, UserIdentifier, DB},
    models::{ErrorResponse, Member, MemberEdit, Sphere},
    Conf,
};

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

/// Edit a member's data using a [`SphereIdentifier`] and [`UserIdentifier`].
///
/// This route requires permissions to edit other users.
///
/// Only a user can change their own avatar, banner, bio and status, moderators can reset them
/// however.
///
/// -----
///
/// ### Example
///
/// ```sh
/// # TODO: add example
/// ```
#[autodoc("/spheres", category = "Members")]
#[patch("/<sphere_identifier>/members/<member_identifier>", data = "<edit>")]
pub async fn edit_member(
    edit: Json<MemberEdit>,
    sphere_identifier: SphereIdentifier,
    member_identifier: UserIdentifier,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Result<Json<Member>, ErrorResponse>> {
    let mut rate_limiter;
    rate_limiter = RateLimiter::new("edit_member", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;
    let sphere = match sphere_identifier {
        SphereIdentifier::ID(id) => Sphere::get_unpopulated(id, &mut db).await,
        SphereIdentifier::Slug(slug) => Sphere::get_unpopulated_slug(slug, &mut db).await,
    }
    .map_err(|err| rate_limiter.add_headers(err))?;
    let mut cache = cache.into_inner();
    let member = match member_identifier {
        UserIdentifier::Me => {
            Member::get(
                session.0.user_id,
                sphere.id,
                Some(session.0.user_id),
                &mut db,
                &mut cache,
            )
            .await
        }
        UserIdentifier::ID(id) => {
            Member::get(id, sphere.id, Some(session.0.user_id), &mut db, &mut cache).await
        }
        UserIdentifier::Username(username) => {
            Member::get_username(
                &username,
                sphere.id,
                Some(session.0.user_id),
                &mut db,
                &mut cache,
            )
            .await
        }
    }
    .map_err(|err| rate_limiter.add_headers(err))?;
    if session.0.user_id != member.user.id && session.0.user_id != sphere.owner_id {
        error!(rate_limiter, FORBIDDEN);
    }
    rate_limiter.wrap_response(Ok(Json(
        Member::edit(
            member.user.id,
            sphere.id,
            edit.into_inner(),
            &mut db,
            Some(session.0.user_id),
            &mut cache,
        )
        .await
        .map_err(|err| rate_limiter.add_headers(err))?,
    )))
}
