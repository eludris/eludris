use rocket_db_pools::Connection;
use todel::{
    http::{Cache, TokenAuth, UserIdentifier, DB},
    models::{ErrorResponse, FetchResponse, File, User},
};

#[autodoc("/users", category = "Users")]
#[get("/<identifier>/avatar?<size>")]
pub async fn get_avatar<'a>(
    identifier: UserIdentifier,
    size: Option<u32>,
    mut db: Connection<DB>,
    cache: Connection<Cache>,
    session: Option<TokenAuth>,
) -> Result<FetchResponse<'a>, ErrorResponse> {
    let user = match identifier {
        UserIdentifier::Me => match session {
            Some(session) => User::get_unfiltered(session.0.user_id, &mut db).await,
            None => Err(error!(UNAUTHORIZED)),
        },
        UserIdentifier::ID(id) => User::get_unfiltered(id, &mut db).await,
        UserIdentifier::Username(username) => {
            User::get_username(
                &username,
                session.map(|s| s.0.user_id),
                &mut db,
                &mut cache.into_inner(),
            )
            .await
        }
    }?;
    match user.avatar {
        Some(avatar) => File::fetch_file(avatar, "avatars", size, &mut db).await,
        None => Err(error!(NOT_FOUND)),
    }
}
