#[macro_use]
extern crate rocket;
#[macro_use]
extern crate todel;

mod cors;
mod database;
mod rate_limit;
mod routes;

use std::env;
#[cfg(test)]
use std::sync::Once;

use anyhow::Context;
use database::DatabaseFairing;
use rocket::{Build, Config, Rocket};
use rocket_db_pools::{deadpool_redis::Pool, sqlx::PgPool, Database};
use routes::*;
use todel::Conf;

#[cfg(test)]
static INIT: Once = Once::new();

#[derive(Database)]
#[database("db")]
pub struct DB(PgPool);

#[derive(Database)]
#[database("cache")]
pub struct Cache(Pool);

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
        .attach(DB::init())
        .attach(Cache::init())
        .attach(cors::Cors)
        .attach(DatabaseFairing)
        .mount("/", get_routes())
        .mount("/messages", messages::get_routes()))
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
