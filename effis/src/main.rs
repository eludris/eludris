#[macro_use]
extern crate rocket;
#[macro_use]
extern crate todel;

mod cors;
mod rate_limit;
mod routes;

use std::{env, path::Path};

use anyhow::Context;

use rocket::{
    data::{Limits, ToByteUnit},
    tokio::sync::Mutex,
    Build, Config, Rocket,
};
use rocket_db_pools::{deadpool_redis::Pool, sqlx::MySqlPool, Database};
use todel::{
    ids::{generate_instance_id, IDGenerator},
    Conf,
};
use tokio::fs;

pub const BUCKETS: [&str; 1] = ["attachments"];

#[derive(Database)]
#[database("db")]
pub struct DB(MySqlPool);

#[derive(Database)]
#[database("cache")]
pub struct Cache(Pool);

fn rocket() -> Result<Rocket<Build>, anyhow::Error> {
    #[cfg(test)]
    {
        env::set_var("ELUDRIS_CONF", "../tests/Eludris.toml");
        dotenvy::dotenv().ok();
        env_logger::try_init().ok();
    }

    let conf = Conf::new_from_env()?;

    let config = Config::figment()
        .merge((
            "port",
            env::var("EFFIS_PORT")
                .unwrap_or_else(|_| "7161".to_string())
                .parse::<u32>()
                .context("Invalid \"EFFIS_PORT\" environment variable")?,
        ))
        .merge((
            "limits",
            Limits::default()
                .limit(
                    "data-form",
                    conf.effis.attachment_file_size.bytes() + 1.mebibytes(), // leeway
                )
                .limit("file", conf.effis.attachment_file_size.bytes()),
        ))
        .merge(("temp_dir", "files"))
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
        .manage(Mutex::new(IDGenerator::new(generate_instance_id())))
        .manage(conf)
        .attach(DB::init())
        .attach(Cache::init())
        .attach(cors::Cors)
        .mount("/", routes::routes())
        .mount("/static/", routes::static_routes()))
}

#[rocket::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:root@localhost:3306/eludris".to_string());

    let pool = MySqlPool::connect(&db_url)
        .await
        .with_context(|| format!("Failed to connect to database on {}", db_url))?;
    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .context("Failed to run migrations")?;

    create_file_dirs().await?;

    let _ = rocket()?
        .launch()
        .await
        .context("Encountered an error while running Rest API")?;

    Ok(())
}

async fn create_file_dirs() -> Result<(), anyhow::Error> {
    try_create_dir("files").await?;
    try_create_dir("files/static").await?;
    for dir in BUCKETS.iter() {
        try_create_dir(format!("files/{dir}")).await?;
    }
    Ok(())
}

async fn try_create_dir(path: impl AsRef<Path>) -> Result<(), anyhow::Error> {
    if !path.as_ref().exists() {
        fs::create_dir(&path)
            .await
            .with_context(|| format!("Failed to create {} directory", path.as_ref().display()))
    } else {
        Ok(())
    }
}
