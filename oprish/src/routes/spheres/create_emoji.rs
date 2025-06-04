use rocket::{serde::json::Json, State};
use rocket_db_pools::{deadpool_redis::redis::AsyncCommands, Connection};
use todel::{
    http::{Cache, SphereIdentifier, TokenAuth, DB},
    ids::IdGenerator,
    models::{Emoji, EmojiCreate, ServerPayload, Sphere},
    Conf,
};
use tokio::sync::Mutex;

use crate::rate_limit::{RateLimitedRouteResponse, RateLimiter};

#[autodoc("/spheres", category = "Emojis")]
#[post("/<identifier>/emojis", data = "<emoji>")]
pub async fn create_emoji(
    emoji: Json<EmojiCreate>,
    identifier: SphereIdentifier,
    conf: &State<Conf>,
    mut cache: Connection<Cache>,
    mut db: Connection<DB>,
    id_generator: &State<Mutex<IdGenerator>>,
    session: TokenAuth,
) -> RateLimitedRouteResponse<Json<Emoji>> {
    let mut rate_limiter = RateLimiter::new("create_emoji", session.0.user_id, conf);
    rate_limiter.process_rate_limit(&mut cache).await?;
    let sphere = match identifier {
        SphereIdentifier::ID(id) => Sphere::get_unpopulated(id, &mut db).await,
        SphereIdentifier::Slug(slug) => {
            Sphere::get_unpopulated_slug(slug.to_string(), &mut db).await
        }
    }
    .map_err(|err| rate_limiter.add_headers(err))?;

    let emoji = sphere
        .add_emoji(
            emoji.into_inner(),
            session.0.user_id,
            &mut *id_generator.lock().await,
            &mut db,
        )
        .await
        .map_err(|err| rate_limiter.add_headers(err))?;

    cache
        .publish::<&str, String, ()>(
            "eludris-events",
            serde_json::to_string(&ServerPayload::EmojiCreate {
                sphere_id: sphere.id,
                emoji: emoji.clone(),
            })
            .unwrap(),
        )
        .await
        .unwrap();

    rate_limiter.wrap_response(Json(emoji))
}
