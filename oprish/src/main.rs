#[macro_use]
extern crate rocket;
#[macro_use]
extern crate todel;

mod cleanup;
mod cors;
mod database;
mod email;
mod rate_limit;
mod routes;

use std::env;
#[cfg(test)]
use std::sync::Once;

use anyhow::Context;
use argon2::Argon2;
use cleanup::ScheduledCleanup;
use database::DatabaseFairing;
use email::EmailFairing;
use rand::{rngs::StdRng, SeedableRng};
use rocket::{Build, Config, Rocket};
use rocket_db_pools::Database;
use todel::{
    http::{Cache, DB},
    ids::IdGenerator,
    Conf,
};
use tokio::sync::Mutex;

use routes::*;

#[cfg(test)]
static INIT: Once = Once::new();

fn rocket() -> Result<Rocket<Build>, anyhow::Error> {
    #[cfg(test)]
    {
        INIT.call_once(|| {
            env::set_current_dir("..").expect("Could not set the current directory");
            env::set_var("ELUDRIS_CONF", "tests/Eludris.toml");
            dotenvy::dotenv().ok();
            env_logger::init();
        });
    }

    let config = Config::figment()
        .merge((
            "port",
            env::var("OPRISH_PORT")
                .unwrap_or_else(|_| "7159".to_string())
                .parse::<u32>()
                .context("Invalid \"OPRISH_PORT\" environment variable")?,
        ))
        .merge((
            "databases.db",
            rocket_db_pools::Config {
                url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                    "postgresql://root:root@localhost:5432/eludris".to_string()
                }),
                min_connections: None,
                max_connections: 1024,
                connect_timeout: 3,
                idle_timeout: None,
            },
        ))
        .merge((
            "databases.cache",
            rocket_db_pools::Config {
                url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
                min_connections: None,
                max_connections: 1024,
                connect_timeout: 3,
                idle_timeout: None,
            },
        ));

    Ok(rocket::custom(config)
        .manage(Conf::new_from_env()?)
        .manage(Mutex::new(StdRng::from_entropy()))
        .manage(Mutex::new(IdGenerator::new()))
        .manage(Argon2::default())
        .attach(DB::init())
        .attach(Cache::init())
        .attach(cors::Cors)
        .attach(DatabaseFairing)
        .attach(EmailFairing)
        .attach(ScheduledCleanup)
        .mount("/", get_routes())
        .mount("/messages", messages::get_routes())
        .mount("/users", users::get_routes())
        .mount("/sessions", sessions::get_routes()))
}

#[rocket::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let _ = rocket()?
        .launch()
        .await
        .context("Encountered an error while running Rest API")?;

    Ok(())
}
