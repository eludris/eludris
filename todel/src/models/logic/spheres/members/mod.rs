mod edit;

use redis::AsyncCommands;
use sqlx::{pool::PoolConnection, Postgres, Row};

use crate::models::{ErrorResponse, Member, User};

impl Member {
    pub async fn get<C: AsyncCommands>(
        id: u64,
        sphere_id: u64,
        requester_id: Option<u64>,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<Self, ErrorResponse> {
        let user = User::get(id, requester_id, db, cache).await?;
        sqlx::query(
            "
            SELECT * 
            FROM members
            WHERE id = $1
            AND sphere_id = $2
            ",
        )
        .bind(id as i64)
        .bind(sphere_id as i64)
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch member {}'s data: {}", id, err);
            error!(SERVER, "Failed to fetch member")
        })?
        .map(|r| Self {
            user,
            sphere_id: r.get::<i64, _>("id") as u64,
            nickname: r.get("nickname"),
            sphere_avatar: r.get::<Option<i64>, _>("sphere_avatar").map(|i| i as u64),
            sphere_banner: r.get::<Option<i64>, _>("sphere_banner").map(|i| i as u64),
            sphere_bio: r.get("sphere_bio"),
            sphere_status: r.get("sphere_status"),
        })
        .ok_or_else(|| error!(NOT_FOUND))
    }

    pub async fn get_username<C: AsyncCommands>(
        username: &str,
        sphere_id: u64,
        requester_id: Option<u64>,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<Self, ErrorResponse> {
        let user = User::get_username(username, requester_id, db, cache).await?;
        sqlx::query(
            "
            SELECT * 
            FROM members
            WHERE id = $1
            AND sphere_id = $2
            ",
        )
        .bind(user.id as i64)
        .bind(sphere_id as i64)
        .fetch_optional(&mut **db)
        .await
        .map_err(|err| {
            log::error!("Couldn't fetch member {}'s data: {}", user.id, err);
            error!(SERVER, "Failed to fetch member")
        })?
        .map(|r| Self {
            user,
            sphere_id: r.get::<i64, _>("id") as u64,
            nickname: r.get("nickname"),
            sphere_avatar: r.get::<Option<i64>, _>("sphere_avatar").map(|i| i as u64),
            sphere_banner: r.get::<Option<i64>, _>("sphere_banner").map(|i| i as u64),
            sphere_bio: r.get("sphere_bio"),
            sphere_status: r.get("sphere_status"),
        })
        .ok_or_else(|| error!(NOT_FOUND))
    }
}
