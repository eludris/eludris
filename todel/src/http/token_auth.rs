use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use rocket_db_pools::Database;

use crate::{
    error,
    models::{ErrorResponse, Secret, Session},
};

use super::DB;

pub struct TokenAuth(pub Session);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for TokenAuth {
    type Error = ErrorResponse;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let pool = DB::fetch(request.rocket()).expect("Could not get the managed pool");
        let mut db = pool
            .acquire()
            .await
            .expect("Failed to acquire database connection");
        let secret = request
            .rocket()
            .state::<Secret>()
            .expect("Could not obtain the managed Secret");
        match request.headers().get_one("Authorization") {
            Some(token) => match Session::validate_token(token, secret, &mut db).await {
                Ok(session) => Outcome::Success(Self(session)),
                Err(err) => Outcome::Failure((Status::Unauthorized, err)),
            },
            None => Outcome::Failure((Status::Unauthorized, error!(UNAUTHORIZED))),
        }
    }
}
