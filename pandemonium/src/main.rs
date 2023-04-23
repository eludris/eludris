mod handle_connection;
mod rate_limit;
mod utils;

use anyhow::Context;
use std::{env, sync::Arc};
use todel::Conf;
use tokio::{net::TcpListener, task};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1".to_string());
    let gateway_address = format!(
        "{}:{}",
        env::var("PANDEMONIUM_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string()),
        env::var("PANDEMONIUM_PORT").unwrap_or_else(|_| "7160".to_string())
    );

    let redis_client = redis::Client::open(redis_url)?;

    let conf = Arc::new(Conf::new_from_env()?);

    let socket = TcpListener::bind(&gateway_address)
        .await
        .with_context(|| format!("Couldn't start a websocket on {}", gateway_address))?;

    log::info!("Gateway started at {}", gateway_address);

    while let Ok((stream, addr)) = socket.accept().await {
        log::debug!("New connection on ip {}", addr);
        let mut pubsub = match redis_client.get_async_connection().await {
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
        let cache = match redis_client.get_async_connection().await {
            Ok(connection) => connection,
            Err(err) => {
                log::warn!("Couldn't get an async connection to redis, {:?}", err);
                continue;
            }
        };
        task::spawn(handle_connection::handle_connection(
            stream,
            addr,
            cache,
            pubsub,
            Arc::clone(&conf),
        ));
    }

    Ok(())
}
