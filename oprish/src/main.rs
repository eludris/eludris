#[macro_use]
extern crate rocket;
#[macro_use]
extern crate todel;

mod cors;
mod rate_limit;
mod routes;

use std::env;

use anyhow::Context;
use rocket::{Build, Config, Rocket};
use rocket_db_pools::Database;
use routes::*;
use todel::Conf;

#[derive(Database)]
#[database("cache")]
pub struct Cache(deadpool_redis::Pool);

fn rocket() -> Result<Rocket<Build>, anyhow::Error> {
    #[cfg(test)]
    {
        env::set_var("ELUDRIS_CONF", "../tests/Eludris.toml");
    }
    dotenv::dotenv().ok();
    env_logger::try_init().ok();

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
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "mysql://root:root@localhost:3306/eludris".to_string()),
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
        .mount("/", get_routes())
        .mount("/messages", messages::get_routes())
        .manage(Conf::new_from_env()?)
        .attach(Cache::init())
        .attach(cors::Cors))
}

#[rocket::main]
async fn main() -> Result<(), anyhow::Error> {
    let _ = rocket()?
        .launch()
        .await
        .context("Encountered an error while running Rest API")?;

    Ok(())
}
