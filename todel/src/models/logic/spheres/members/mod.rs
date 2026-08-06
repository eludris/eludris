mod edit;

use redis::AsyncCommands;
use sqlx::{pool::PoolConnection, Postgres, Row};

use crate::models::{ErrorResponse, Member, MemberBase, SpherePermissions, SphereRole, User};

impl Member {
    // TODO: this is super stupid and juvenile why are we requiring the user object as the base???
    // why not have it be id-based at origin then use that as the base for our other silly
    // manoeuvres
    pub async fn get_user(
        user: User,
        sphere_id: u64,
        db: &mut PoolConnection<Postgres>,
    ) -> Result<Self, ErrorResponse> {
        // TODO: what?? (why are we fetching every role instead of just the user's roles??)
        let roles = sqlx::query!(
            "
            SELECT *
            FROM roles
            WHERE id = $1
            AND is_deleted = FALSE
            ",
            user.id as i64
        )
        .fetch_all(&mut **db)
        .await
        .map_err(|err| {
            log::error!(
                "Failed to get member {}'s roles from database for sphere {}: {}",
                user.id,
                sphere_id,
                err
            );
            error!(SERVER, "Failed to fetch member")
        })?
        .into_iter()
        .map(|r| SphereRole {
            id: r.id as u64,
            sphere_id: r.sphere_id as u64,
            position: r.position as u32,
            name: r.name,
            colour: r.colour as u32,
            allowed_permissions: SpherePermissions::from_bits(r.allowed as u64),
            denied_permissions: SpherePermissions::from_bits(r.allowed as u64),
        })
        .collect();

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
            log::error!(
                "Couldn't fetch member {}'s data for sphere {}: {}",
                user.id,
                sphere_id,
                err
            );
            error!(SERVER, "Failed to fetch member")
        })?
        .map(|r| Self {
            data: MemberBase {
                user,
                sphere_id: r.get::<i64, _>("id") as u64,
                nickname: r.get("nickname"),
                sphere_avatar: r.get::<Option<i64>, _>("sphere_avatar").map(|i| i as u64),
                sphere_banner: r.get::<Option<i64>, _>("sphere_banner").map(|i| i as u64),
                sphere_bio: r.get("sphere_bio"),
                sphere_status: r.get("sphere_status"),
            },
            roles,
        })
        .ok_or_else(|| error!(NOT_FOUND))
    }

    pub async fn get<C: AsyncCommands>(
        id: u64,
        sphere_id: u64,
        requester_id: Option<u64>,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<Self, ErrorResponse> {
        let user = User::get(id, requester_id, db, cache).await?;
        Self::get_user(user, sphere_id, db).await
    }

    pub async fn get_username<C: AsyncCommands>(
        username: &str,
        sphere_id: u64,
        requester_id: Option<u64>,
        db: &mut PoolConnection<Postgres>,
        cache: &mut C,
    ) -> Result<Self, ErrorResponse> {
        let user = User::get_username(username, requester_id, db, cache).await?;
        Self::get_user(user, sphere_id, db).await
    }
}
