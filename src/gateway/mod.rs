//! The Eludris gateway, also called Pandemonium.

use rocket::futures::{SinkExt, StreamExt};
use rocket::tokio::net::{TcpListener, TcpStream};
use rocket::tokio::{select, sync::Mutex, task, time};
use std::time::{Duration, Instant};
use std::{env, net::SocketAddr, sync::Arc};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::models::client::{Client, Clients};

/// The duration it takes for a connection to be inactive in for it to be regarded as zombified and
/// disconnected.
const TIMEOUT_DURATION: Duration = Duration::from_secs(20);

/// A simple function that check's if a client's last ping was over TIMEOUT_DURATION seconds ago and
/// closes the gateway connection if so.
async fn check_connection(last_ping: Arc<Mutex<Instant>>) {
    let mut interval = time::interval(TIMEOUT_DURATION);
    loop {
        if Instant::now().duration_since(*last_ping.lock().await) > TIMEOUT_DURATION {
            break;
        }
        interval.tick().await;
    }
}

/// A function that handles one client connecting and disconnecting.
async fn handle_connection(addr: SocketAddr, stream: TcpStream, clients: Clients) {
    let socket = accept_async(stream)
        .await
        .expect("Couldn't accept the socket stream.");

    let (tx, mut rx) = socket.split();
    let tx = Arc::new(Mutex::new(tx));

    let last_ping = Arc::new(Mutex::new(Instant::now()));

    {
        let mut clients = clients.lock().await;
        clients.push(Client {
            addr,
            tx: tx.clone(),
        });
    }

    let handle_rx = async {
        while let Some(msg) = rx.next().await {
            log::debug!("New gateway message:\n{:#?}", msg);
            match msg {
                Ok(data) => match data {
                    Message::Ping(x) => {
                        let mut last_ping = last_ping.lock().await;
                        if Instant::now().duration_since(*last_ping) < Duration::from_secs(2) {
                            // A simple form of gateawy ratelimiting.
                            log::info!("Disconnected a client: {}, reason: Ping ratelimit", addr);
                            break;
                        }
                        *last_ping = Instant::now();
                        tx.lock()
                            .await
                            .send(Message::Pong(x))
                            .await
                            .expect("Couldn't send pong");
                    }
                    _ => log::debug!("Unsupported Gateway message type."),
                },
                Err(_) => break,
            }
        }
    };

    select! {
        _ = check_connection(last_ping.clone()) => {},
        _ = handle_rx => {},
    }

    log::info!("Someone disconnected");

    {
        let mut clients = clients.lock().await;
        let idex = clients
            .iter()
            .position(|c| c.addr == addr)
            .expect("Client not found");
        // We use `swap_remove` instead of `remove` here because it has O(1) complexity and we
        // don't really care about the orders of the clients here.
        clients.swap_remove(idex);
    }
}

/// A function that starts & handles the gatway.
pub async fn start(clients: Clients) {
    let gateway_address = format!(
        "{}:{}",
        env::var("GATEWAY_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string()),
        env::var("GATEWAY_PORT").unwrap_or_else(|_| "9000".to_string())
    );
    let socket = TcpListener::bind(&gateway_address)
        .await
        .unwrap_or_else(|_| panic!("Couldn't start a websocket on {}", gateway_address));
    log::info!("Gateway started at {}", gateway_address);

    while let Ok((stream, addr)) = socket.accept().await {
        log::info!("New connection on ip {}", addr);
        task::spawn(handle_connection(addr, stream, clients.clone()));
    }
}
