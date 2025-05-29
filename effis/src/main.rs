#[macro_use]
extern crate rocket;
#[macro_use]
extern crate todel;

mod cors;
mod rate_limit;
mod routes;

#[cfg(test)]
use std::sync::Once;
use std::{env, fs, path::Path};

use anyhow::Context;

use reqwest::Client;
use rocket::{
    data::{Limits, ToByteUnit},
    tokio::sync::Mutex,
    Build, Config, Rocket,
};
use rocket_db_pools::Database;
use todel::{
    http::{Cache, DB},
    ids::IdGenerator,
    Conf,
};

pub const BUCKETS: [&str; 8] = [
    "attachments",
    "avatars",
    "banners",
    "sphere-icons",
    "sphere-banners",
    "member-avatars",
    "member-banners",
    "emojis",
];

#[cfg(test)]
static INIT: Once = Once::new();

fn rocket() -> Result<Rocket<Build>, anyhow::Error> {
    #[cfg(test)]
    {
        INIT.call_once(|| {
            env::set_current_dir("..").expect("Could not set the current directory");
            env::set_var("ELUDRIS_CONF", "tests/Eludris.toml");
            create_file_dirs().expect("Could not create necessary file directories");
            dotenvy::dotenv().ok();
            env_logger::init();
        });
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
        .manage(Mutex::new(IdGenerator::new()))
        .manage(conf)
        .manage(Client::new())
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

    create_file_dirs()?;

    let _ = rocket()?
        .launch()
        .await
        .context("Encountered an error while running Rest API")?;

    Ok(())
}

fn create_file_dirs() -> Result<(), anyhow::Error> {
    try_create_dir("files")?;
    try_create_dir("files/static")?;
    for dir in BUCKETS.iter() {
        try_create_dir(format!("files/{dir}"))?;
    }
    Ok(())
}

fn try_create_dir(path: impl AsRef<Path>) -> Result<(), anyhow::Error> {
    if !path.as_ref().exists() {
        fs::create_dir(&path)
            .with_context(|| format!("Failed to create {} directory", path.as_ref().display()))
    } else {
        Ok(())
    }
}
