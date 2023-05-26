use rocket::{
    fairing::{Fairing, Info, Kind, Result},
    Build, Rocket,
};
use rocket_db_pools::Database;

use crate::DB;

pub struct DatabaseFairing;

#[rocket::async_trait]
impl Fairing for DatabaseFairing {
    fn info(&self) -> Info {
        Info {
            name: "Handle database migrations & setup",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> Result {
        if let Some(db) = DB::fetch(&rocket) {
            if let Err(err) = sqlx::migrate!("../migrations").run(&db.0).await {
                log::error!("Could not run migrations: {}", err);
                Err(rocket)
            } else {
                Ok(rocket)
            }
        } else {
            log::error!("Could not obtain the database for migrations");
            Err(rocket)
        }
    }
}
