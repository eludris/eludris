mod handle_connection;
mod rate_limit;
mod utils;

#[cfg(test)]
use std::sync::Once;
use std::{env, sync::Arc};

use anyhow::Context;
use sqlx::{pool::PoolOptions, Pool, Postgres};
use todel::{models::Secret, Conf};
use tokio::{net::TcpListener, sync::Mutex, task};

#[cfg(test)]
static INIT: Once = Once::new();

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    #[cfg(test)]
    INIT.call_once(|| {
        env::set_current_dir("..").expect("Could not set the current directory");
        env::set_var("ELUDRIS_CONF", "tests/Eludris.toml");
        dotenvy::dotenv().ok();
        env_logger::init();
    });
    #[cfg(not(test))]
    {
        dotenvy::dotenv().ok();
        env_logger::init();
    }

    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://root@127.0.0.1:5432/eludris".to_string());
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1".to_string());
    let gateway_address = format!(
        "{}:{}",
        env::var("PANDEMONIUM_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string()),
        env::var("PANDEMONIUM_PORT").unwrap_or_else(|_| "7160".to_string())
    );

    let client = redis::Client::open(redis_url)?;
    let cache = Arc::new(Mutex::new(
        client
            .get_async_connection()
            .await
            .context("Couldn't get an async connection to redis")?,
    ));
    // the max connections is to stay consistent with oprish and effis even though postgresql
    // will most likely not support this many connections at once.
    let pool: Pool<Postgres> = PoolOptions::new()
        .max_connections(1024)
        .connect(&db_url)
        .await
        .context("Couldn't establish a database pool")?;
    let pool = Arc::new(pool);
    let secret = Arc::new(Secret::try_get(&pool).await.context(
        "Couldn't get instance secret. Make sure oprish is run atleast once before pandemonium",
    )?);

    let conf = Arc::new(Conf::new_from_env()?);

    let socket = TcpListener::bind(&gateway_address)
        .await
        .with_context(|| format!("Couldn't start a websocket on {}", gateway_address))?;

    log::info!("Gateway started at {}", gateway_address);

    while let Ok((stream, addr)) = socket.accept().await {
        log::debug!("New connection on ip {}", addr);
        let mut pubsub = match client.get_async_connection().await {
            Ok(connection) => connection.into_pubsub(),
            Err(err) => {
                log::warn!("Couldn't get an async connection to redis, {:?}", err);
                continue;
            }
        };
        if let Err(err) = pubsub.subscribe("oprish-events").await {
            log::warn!("Couldn't subscribe to oprish-events: {:?}", err);
            continue;
        }
        task::spawn(handle_connection::handle_connection(
            stream,
            addr,
            Arc::clone(&cache),
            pubsub,
            Arc::clone(&pool),
            Arc::clone(&conf),
            Arc::clone(&secret),
        ));
        log::trace!("Spawned connection handling task for {}", addr);
    }

    Ok(())
}
