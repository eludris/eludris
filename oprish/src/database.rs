use rand::rngs::StdRng;
use rocket::{
    fairing::{Fairing, Info, Kind, Result},
    Build, Rocket,
};
use rocket_db_pools::Database;
use todel::models::Secret;
use tokio::sync::Mutex;

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

    // https://github.com/SergioBenitez/Rocket/issues/1876
    async fn on_ignite(&self, rocket: Rocket<Build>) -> Result {
        if let Some(db) = DB::fetch(&rocket) {
            if let Err(err) = sqlx::migrate!("../migrations").run(&db.0).await {
                log::error!("Could not run migrations: {}", err);
                return Err(rocket);
            }
            let secret = Secret::get(
                &db.0,
                &mut *rocket.state::<Mutex<StdRng>>().unwrap().lock().await,
            )
            .await;
            // Isolated if statement to avoid having rocket borrowed
            if let Ok(secret) = secret {
                Ok(rocket.manage(secret))
            } else {
                Err(rocket)
            }
        } else {
            log::error!("Could not obtain the database for migrations");
            Err(rocket)
        }
    }
}
